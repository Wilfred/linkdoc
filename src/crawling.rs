use crossbeam_channel::{unbounded, Receiver, Sender};
use crossbeam_utils::Backoff;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use url::Url;

use crate::fetching::{fetch_all_urls, url_status, UrlError};

pub struct Crawler {
    active_count: Arc<Mutex<i32>>,
    url_states: Receiver<Result<Url, UrlError>>,
}

impl Iterator for Crawler {
    type Item = Result<Url, UrlError>;

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
    url_s: Sender<String>,
    url_r: Receiver<String>,
    visited: Arc<Mutex<HashSet<String>>>,
    active_count: Arc<Mutex<i32>>,
    url_states: Sender<Result<Url, UrlError>>,
) {
    loop {
        match url_r.try_recv() {
            Ok(current) => {
                {
                    let mut active_count = active_count.lock().unwrap();
                    *active_count += 1;
                    assert!(*active_count <= CRAWL_THREADS);
                }

                // TODO: we are fetching the URL twice, which is silly.
                let state = url_status(domain, &current);

                // Fetch accessible URLs on the same domain and crawl them too.
                if let Ok(ref url) = state.clone() {
                    if url.domain() == Some(domain) {
                        // Lock `visited` and see if we've already visited these discovered URLs.
                        let mut visited = visited.lock().unwrap();

                        for new_url in fetch_all_urls(url) {
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

                url_states.send(state).unwrap();
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

/// Starting at start_url, recursively iterate over all the URLs which match
/// the domain, and return an iterator of their URL status.
pub fn crawl(domain: &str, start_url: &Url) -> Crawler {
    let active_count = Arc::new(Mutex::new(0));

    let mut visited = HashSet::with_capacity(1);
    visited.insert(start_url.as_str().into());
    let visited = Arc::new(Mutex::new(visited));

    let (url_state_s, url_state_r) = unbounded();
    let (visit_s, visit_r) = unbounded();
    visit_s.send(start_url.as_str().into()).unwrap();

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
