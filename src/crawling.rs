use std::collections::HashSet;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use url::Url;

use fetching::{UrlState, url_status, fetch_all_urls};

pub struct Crawler {
    // TODO: use a proper deque.
    to_visit: Arc<Mutex<Vec<String>>>,

    active_count: Arc<Mutex<i32>>,

    url_states: Receiver<UrlState>,
}

impl Iterator for Crawler {
    type Item = UrlState;

    fn next(&mut self) -> Option<UrlState> {
        loop {
            match self.url_states.try_recv() {
                // If there's currently something in the channel, return
                // it.
                Ok(state) => return Some(state),

                Err(_) => {
                    let to_visit_val = self.to_visit.lock().unwrap();
                    let active_count_val = self.active_count.lock().unwrap();

                    if to_visit_val.is_empty() && *active_count_val == 0 {
                        // We're done, no values left.
                        return None
                    } else {
                        // The channel is currently empty, but we will
                        // more values later.
                        continue
                    }
                }
            }
        }
    }
}

const THREADS: i32 = 10;

fn crawl_worker_thread(domain: &str,
                       to_visit: Arc<Mutex<Vec<String>>>,
                       visited: Arc<Mutex<HashSet<String>>>,
                       active_count: Arc<Mutex<i32>>,
                       url_states: Sender<UrlState>) {
    loop {
        let current;
        {
            // Lock `to_visit` vector, and try to get an URL to visit.
            let mut to_visit_val = to_visit.lock().unwrap();
            let mut active_count_val = active_count.lock().unwrap();
            if to_visit_val.is_empty() {
                // If there are requests still in flight, we might
                // get more work in the future.
                if *active_count_val > 0 {
                    continue
                } else {
                    // There won't be any more URLs to visit, so terminate this thread.
                    break
                }
            };
            current = to_visit_val.pop().unwrap();
            *active_count_val += 1;
            assert!(*active_count_val <= THREADS);
        }

        {
            // Lock `visited` and see if we've already visited this URL.
            let mut visited_val = visited.lock().unwrap();
            if visited_val.contains(&current) {
                // Nothing left to do here, so decrement count.
                let mut active_count_val = active_count.lock().unwrap();
                *active_count_val -= 1;
                continue
            } else {
                visited_val.insert(current.to_owned());
            }
        }

        // TODO: we are fetching the URL twice, which is silly.
        let state = url_status(&domain, &current);

        // If it's accessible and it's on the same domain:
        if let UrlState::Accessible(ref url) = state.clone() {
            if url.domain() == Some(&domain) {
                // then fetch it and append all the URLs found.
                let new_urls = fetch_all_urls(&url);

                let mut to_visit_val = to_visit.lock().unwrap();
                for new_url in new_urls {
                    to_visit_val.push(new_url);
                }
            }
        }

        {
            // This thread is now done, so decrement the count.
            let mut active_count_val = active_count.lock().unwrap();
            *active_count_val -= 1;
            assert!(*active_count_val >= 0);
        }

        url_states.send(state).unwrap();
    }
}

/// Starting at start_url, recursively iterate over all the URLs which match
/// the domain, and return an iterator of their URL status.
pub fn crawl(domain: &str, start_url: &Url) -> Crawler {
    let to_visit = Arc::new(Mutex::new(vec![start_url.serialize()]));
    let active_count = Arc::new(Mutex::new(0));
    let visited = Arc::new(Mutex::new(HashSet::new()));

    let (tx, rx) = channel();

    let crawler = Crawler {
        to_visit: to_visit.clone(),
        active_count: active_count.clone(),
        url_states: rx,
    };

    for _ in 0..THREADS {
        let domain = domain.to_owned();
        let to_visit = to_visit.clone();
        let visited = visited.clone();
        let active_count = active_count.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            crawl_worker_thread(&domain, to_visit, visited, active_count, tx);
        });
    }

    crawler
}
