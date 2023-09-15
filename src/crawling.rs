use crossbeam_channel::{unbounded, Receiver, Sender};
use crossbeam_utils::Backoff;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use tokio_stream::Stream;
use url::Url;

use crate::fetching::{url_fetch, url_status, UrlError};
use crate::parsing::get_parsed_urls;

pub struct Crawler {
    active_count: Arc<Mutex<i32>>,
    url_states: Receiver<Result<(), UrlError>>,
}

impl Iterator for Crawler {
    type Item = Result<(), UrlError>;

    fn next(&mut self) -> Option<Self::Item> {
        let backoff = Backoff::new();
        loop {
            match self.url_states.try_recv() {
                // If there's currently something in the channel, return
                // it.
                Ok(state) => return Some(state),

                Err(_) => {
                    {
                        let active_count = self.active_count.lock().unwrap();
                        if *active_count == 0 {
                            // We're done, no values left.
                            return None;
                        }
                    }
                    // The channel is currently empty, but we will
                    // have more values later.
                    backoff.snooze();
                }
            }
        }
    }
}

const CRAWL_THREADS: i32 = 10;

/// Read URLs from the `url_r` channel, and write url states to the
/// `url_states` channel. Write new URLs discovered back to the
/// `url_s` channel.
fn crawl_worker_thread(
    domain: &str,
    url_s: Sender<Url>,
    url_r: Receiver<Url>,
    visited: Arc<Mutex<HashSet<Url>>>,
    active_count: Arc<Mutex<i32>>,
    url_states: Sender<Result<(), UrlError>>,
) {
    loop {
        match url_r.try_recv() {
            Ok(current) => {
                {
                    let mut active_count = active_count.lock().unwrap();
                    *active_count += 1;
                    assert!(*active_count <= CRAWL_THREADS);
                }

                let state = url_status(&current);

                // Fetch accessible URLs on the same domain and crawl them too.
                if let Ok(ref html_src) = state {
                    if current.domain() == Some(domain) {
                        // Lock `visited` and see if we've already visited these discovered URLs.
                        let mut visited = visited.lock().unwrap();

                        let parsed_urls = get_parsed_urls(html_src, domain);
                        for malformed_url in parsed_urls.malformed_urls {
                            url_states
                                .send(Err(UrlError::Malformed(malformed_url)))
                                .unwrap();
                        }

                        for new_url in parsed_urls.urls {
                            if !visited.contains(&new_url) {
                                visited.insert(new_url.clone());
                                url_s.send(new_url).unwrap();
                            }
                        }
                    }
                }

                {
                    // This thread is now done, so decrement the count.
                    let mut active_count = active_count.lock().unwrap();
                    *active_count -= 1;
                    assert!(*active_count >= 0);
                }

                url_states.send(state.map(|_| ())).unwrap();
            }
            Err(_) => {
                let active_count = active_count.lock().unwrap();
                // Nothing in the channel for us to do.
                // If there are requests still in flight, we might
                // get more work in the future.
                if *active_count > 0 {
                    // snooze
                } else {
                    // There won't be any more URLs to visit, so terminate this thread.
                    break;
                }
            }
        }
    }
}

fn spawn_crawl_tasks(
    domain: &str,
    url_s: Sender<Url>,
    url_r: Receiver<Url>,
    visited: Arc<Mutex<HashSet<Url>>>,
    active_count: Arc<Mutex<i32>>,
    url_states: Sender<Result<(), UrlError>>,
) {
    loop {
        match url_r.try_recv() {
            Ok(current) => {
                let domain = domain.to_owned();
                let url_s = url_s.clone();
                let visited = visited.clone();
                let active_count = active_count.clone();
                let url_states = url_states.clone();

                tokio::spawn(async move {
                    {
                        let mut active_count = active_count.lock().unwrap();
                        *active_count += 1;
                    }

                    let state = url_fetch(&current).await;

                    if let Ok(ref html_src) = state {
                        // Fetch accessible URLs on the same domain and crawl them too.
                        if current.domain() == Some(&domain) {
                            // Lock `visited` and see if we've already visited these discovered URLs.
                            let mut visited = visited.lock().unwrap();

                            let parsed_urls = get_parsed_urls(html_src, &domain);
                            for malformed_url in parsed_urls.malformed_urls {
                                url_states
                                    .send(Err(UrlError::Malformed(malformed_url)))
                                    .unwrap();
                            }

                            for new_url in parsed_urls.urls {
                                if !visited.contains(&new_url) {
                                    visited.insert(new_url.clone());
                                    url_s.send(new_url).unwrap();
                                }
                            }
                        }
                    }

                    {
                        // This task is now done, so decrement the count.
                        let mut active_count = active_count.lock().unwrap();
                        *active_count -= 1;
                        assert!(*active_count >= 0);
                    }

                    url_states.send(state.map(|_| ())).unwrap();
                });
            }
            Err(_) => {
                let active_count = active_count.lock().unwrap();
                // Nothing in the channel for us to do.
                // If there are requests still in flight, we might
                // get more work in the future.
                if *active_count > 0 {
                    // snooze
                } else {
                    // There won't be any more URLs to visit, so terminate this thread.
                    break;
                }
            }
        }
    }
}

pub struct AsyncCrawler {
    active_count: Arc<Mutex<i32>>,
    url_states: Receiver<Result<(), UrlError>>,
}

impl Stream for AsyncCrawler {
    type Item = Result<(), UrlError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.url_states.try_recv() {
            // If there's currently something in the channel, return
            // it.
            Ok(state) => return Poll::Ready(Some(state)),

            Err(_) => {
                {
                    let active_count = self.active_count.lock().unwrap();
                    if *active_count == 0 {
                        // We're done, no values left.
                        return Poll::Ready(None);
                    }
                }

                // The channel is currently empty, but we will
                // have more values later.
                return Poll::Pending;
            }
        }
    }
}

pub async fn async_crawl(domain: &str, start_url: &Url) -> AsyncCrawler {
    let active_count = Arc::new(Mutex::new(0));

    let mut visited = HashSet::with_capacity(1);
    visited.insert(start_url.clone());
    let visited = Arc::new(Mutex::new(visited));

    let (url_state_s, url_state_r) = unbounded();

    let (visit_s, visit_r) = unbounded();
    visit_s.send(start_url.to_owned()).unwrap();

    let domain = domain.to_owned();
    let visited = visited.clone();
    let active_count2 = active_count.clone();
    let url_state_s = url_state_s.clone();
    let visit_r = visit_r.clone();
    let visit_s = visit_s.clone();
    tokio::spawn(async move {
        spawn_crawl_tasks(
            &domain,
            visit_s,
            visit_r,
            visited,
            active_count2,
            url_state_s,
        )
    });

    // TODO: race condition. If we start iterating the AsyncCrawler
    // too fast, we could terminate before we have our first result.
    AsyncCrawler {
        active_count,
        url_states: url_state_r,
    }
}

/// Starting at start_url, recursively iterate over all the URLs which match
/// the domain, and return an iterator of their URL status.
pub fn crawl(domain: &str, start_url: &Url) -> Crawler {
    let active_count = Arc::new(Mutex::new(0));

    let mut visited = HashSet::with_capacity(1);
    visited.insert(start_url.clone());
    let visited = Arc::new(Mutex::new(visited));

    let (url_state_s, url_state_r) = unbounded();

    let (visit_s, visit_r) = unbounded();
    visit_s.send(start_url.to_owned()).unwrap();

    let crawler = Crawler {
        active_count: active_count.clone(),
        url_states: url_state_r,
    };

    for _ in 0..CRAWL_THREADS {
        let domain = domain.to_owned();
        let visited = visited.clone();
        let active_count = active_count.clone();
        let url_state_s = url_state_s.clone();
        let visit_r = visit_r.clone();
        let visit_s = visit_s.clone();

        thread::spawn(move || {
            crawl_worker_thread(
                &domain,
                visit_s,
                visit_r,
                visited,
                active_count,
                url_state_s,
            );
        });
    }

    crawler
}
