use std::io::stdout;
use std::io::Write;
use std::collections::HashSet;
use std::sync::Mutex;
use fetching::{UrlState, url_status, fetch_all_urls};

/// Starting at start_url, recursively iterate over all the URLs which match
/// this domain, and return their URL status.
pub struct Crawler {
    domain: String,
    
    visited: Mutex<HashSet<String>>,
    // TODO: use a proper deque.
    to_visit: Mutex<Vec<String>>,
}

/// Return a string that is exactly the size specified. If it's too
/// big, truncate, otherwise pad with spaces.
fn exact_size(s: &str, size: i64) -> String {
    let mut result = s.to_owned();
    let size_delta = result.chars().count() as i64 - size;
    
    if size_delta > 0 {
        result.truncate(size as usize);
    } else {
        for _ in 0..size_delta.abs() {
            result.push(' ');
        }
    }
    result
}

impl Iterator for Crawler {
    type Item = UrlState;

    fn next(&mut self) -> Option<UrlState> {
        let mut to_visit = self.to_visit.lock().unwrap();
        let mut visited = self.visited.lock().unwrap();
        
        while !to_visit.is_empty() {
            let current = to_visit.pop().unwrap();
            
            if !visited.contains(&current) {
                visited.insert(current.to_owned());

                // Ideally we wouldn't be so noisy. However, it's not
                // possible to do timeouts with Hyper:
                // https://github.com/hyperium/hyper/issues/315
                // so it's better to see what's going on than just hang.
                let short_url = exact_size(&current, 60);
                print!("Checked {}, next: {}\r", visited.len(), &short_url);
                stdout().flush().unwrap();
                
                let state = url_status(&self.domain, &current);
                // TODO: we are fetching the URL twice, which is silly.

                // If it's accessible and it's on the same domain:
                if let UrlState::Accessible(ref url) = state.clone() {
                    if url.domain() == Some(&self.domain) {
                        // then fetch it and append all the URLs found.
                        for new_url in fetch_all_urls(&url) {
                            to_visit.push(new_url);
                        }
                    }
                }

                return Some(state);
            }
        }
        None
    }
}

pub fn crawl(domain: &str, start_url: &str) -> Crawler {
    let to_visit = vec![start_url.to_owned()];
    
    Crawler {
        domain: domain.to_owned(),
        visited: Mutex::new(HashSet::new()),
        to_visit: Mutex::new(to_visit)
    }
}
