use std::collections::HashSet;
use fetching::{UrlState, url_status, fetch_all_urls};

/// Starting at start_url, recursively visit all the URLs which match
/// domain, and return their URL status.
pub fn crawl(domain: &str, start_url: &str) -> Vec<UrlState> {
    let mut crawled: HashSet<String> = HashSet::new();
    
    // TODO: use a proper deque.
    let mut to_visit = Vec::new();
    to_visit.push(start_url.to_owned());

    let mut url_states = Vec::new();

    while !to_visit.is_empty() {
        // TODO: this is LIFO and we should use FIFO.
        let current = to_visit.pop().unwrap();

        if !crawled.contains(&current) {
            crawled.insert(current.to_owned());
            
            let state = url_status(domain, &current);
            println!("{}", state);
            url_states.push(state.clone());

            // TODO: we are fetching the URL twice, which is silly.

            // If it's accessible and it's on the same domain:
            if let UrlState::Accessible(ref url) = state {
                if url.domain() == Some(domain) {
                    // then fetch it and append all the URLs found.
                    for new_url in fetch_all_urls(&url) {
                        to_visit.push(new_url);
                    }
                }
            }
        }
    }

    url_states
}
