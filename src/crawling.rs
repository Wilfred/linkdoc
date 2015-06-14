use std::collections::HashSet;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use fetching::{UrlState, url_status, fetch_all_urls};

pub struct Crawler {
    // TODO: use a proper deque.
    to_visit: Arc<Mutex<Vec<String>>>,

    active_threads: Arc<Mutex<i32>>,

    url_states: Receiver<UrlState>,
}

impl Iterator for Crawler {
    type Item = UrlState;

    fn next(&mut self) -> Option<UrlState> {
        loop {
            match self.url_states.recv() {
                // If there's currently something in the channel, return
                // it.
                Ok(state) => return Some(state),
                
                Err(_) => {
                    let to_visit_val = self.to_visit.lock().unwrap();
                    let active_threads_val = self.active_threads.lock().unwrap();

                    if to_visit_val.is_empty() && *active_threads_val == 0 {
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

/// Starting at start_url, recursively iterate over all the URLs which match
/// the domain, and return an iterator of their URL status.
pub fn crawl(domain: &str, start_url: &str) -> Crawler {
    let to_visit = Arc::new(Mutex::new(vec![start_url.to_owned()]));
    let active_threads = Arc::new(Mutex::new(0));
    let visited = Arc::new(Mutex::new(HashSet::new()));

    let (tx, rx) = channel();

    let crawler = Crawler {
        to_visit: to_visit.clone(),
        active_threads: active_threads.clone(),
        url_states: rx,
    };

    for _ in 0..THREADS {
        let domain = domain.to_owned();
        let to_visit = to_visit.clone();
        let visited = visited.clone();
        let active_threads = active_threads.clone();
        let tx = tx.clone();
        
        thread::spawn(move || {
            loop {
                // Lock `to_visit` vector, and try to get an URL to visit.
                let mut to_visit_val = to_visit.lock().unwrap();
                let mut active_threads_val = active_threads.lock().unwrap();
                if to_visit_val.is_empty() {
                    // If there are requests still in flight, we might
                    // get more work in the future.
                    if *active_threads_val > 0 {
                        continue
                    } else {
                        // There won't be any more URLs to visit, so terminate this thread.
                        break
                    }
                };
                let current = to_visit_val.pop().unwrap();
                *active_threads_val += 1;
                drop(to_visit_val);
                assert!(*active_threads_val <= THREADS);
                drop(active_threads_val);

                // Lock `visited` and see if we've already visited this URL.
                let mut visited_val = visited.lock().unwrap();
                if visited_val.contains(&current) {
                    continue
                } else {
                    visited_val.insert(current.to_owned());
                }
                drop(visited_val);

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

                // This thread is now done, so decrement the count.
                let mut active_threads_val = active_threads.lock().unwrap();
                *active_threads_val -= 1;
                assert!(*active_threads_val >= 0);
                drop(active_threads_val);

                tx.send(state).unwrap();
            }
        });
    }

    crawler
}
