use reqwest::StatusCode;
use std::fmt;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use url::{ParseError, Url};
use colored::*;

use crate::parsing;

#[derive(Debug, Clone)]
pub enum UrlState {
    Accessible(Url),
    BadStatus(Url, StatusCode),
    ConnectionFailed(Url),
    TimedOut(Url),
    Malformed(String),
}

impl fmt::Display for UrlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tick = "✔".green();
        let cross = "✘".red();
        match *self {
            UrlState::Accessible(ref url) => format!("{} {}", tick, url).fmt(f),
            UrlState::BadStatus(ref url, ref status) => {
                format!("{} {} ({})", cross, url, status).fmt(f)
            }
            UrlState::ConnectionFailed(ref url) => {
                format!("{} {} (connection failed)", cross, url).fmt(f)
            }
            UrlState::TimedOut(ref url) => format!("{} {} (timed out)", cross, url).fmt(f),
            UrlState::Malformed(ref url) => format!("{} {} (malformed)", cross, url).fmt(f),
        }
    }
}

fn build_url(domain: &str, path: &str) -> Result<Url, ParseError> {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string)?;
    base_url.join(path)
}

const TIMEOUT_SECS: u64 = 10;

pub fn url_status(domain: &str, path: &str) -> UrlState {
    match build_url(domain, path) {
        Ok(url) => {
            let (tx, rx) = channel();
            let request_tx = tx.clone();
            let url2 = url.clone();

            // Try to do the request.
            thread::spawn(move || {
                let response = reqwest::get(url.as_str());

                let _ = request_tx.send(match response {
                    Ok(response) => {
                        if response.status().is_success() {
                            UrlState::Accessible(url)
                        } else {
                            // TODO: allow redirects unless they're circular
                            UrlState::BadStatus(url, response.status())
                        }
                    }
                    Err(_) => UrlState::ConnectionFailed(url),
                });
            });

            // Send a timeout down the channel after a delay.
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(TIMEOUT_SECS));
                let _ = tx.send(UrlState::TimedOut(url2));
            });

            // Take whichever value arrives in the channel first.
            rx.recv().unwrap()
        }
        Err(_) => UrlState::Malformed(path.to_owned()),
    }
}

pub fn fetch_url(url: &Url) -> String {
    // Creating an outgoing request.
    let mut res = reqwest::get(url.as_str()).expect("could not fetch URL");

    // Read the body.
    match res.text() {
        Ok(body) => body,
        // TODO: handle malformed data more gracefully.
        Err(_) => String::new(),
    }
}

/// Fetch the requested URL, and return a list of all the URLs on the
/// page. We deliberately return strings because we're also interested
/// in malformed URLs.
pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    parsing::get_urls(&html_src)
}
