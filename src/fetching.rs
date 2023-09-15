use colored::*;
use crossbeam_channel::{select, unbounded};
use reqwest::StatusCode;
use std::fmt;
use std::thread;
use std::time::Duration;
use url::{ParseError, Url};

use crate::parsing;

#[derive(Debug, Clone)]
pub enum UrlError {
    BadStatus(Url, StatusCode),
    InvalidText(Url),
    ConnectionFailed(Url),
    TimedOut(Url),
    Malformed(String),
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cross = "✘".red();
        match *self {
            UrlError::BadStatus(ref url, ref status) => {
                format!("{} {} ({})", cross, url, status).fmt(f)
            }
            UrlError::ConnectionFailed(ref url) => {
                format!("{} {} (connection failed)", cross, url).fmt(f)
            }
            UrlError::InvalidText(ref url) => format!("{} {} (invalid text)", cross, url).fmt(f),
            UrlError::TimedOut(ref url) => format!("{} {} (timed out)", cross, url).fmt(f),
            UrlError::Malformed(ref url) => format!("{} {} (malformed)", cross, url).fmt(f),
        }
    }
}

fn build_url(domain: &str, path: &str) -> Result<Url, ParseError> {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string)?;
    base_url.join(path)
}

const TIMEOUT_SECS: u64 = 10;

pub fn url_status(url: &Url) -> Result<String, UrlError> {
    let (s, r) = unbounded();
    let url = url.clone();
    let url2 = url.clone();

    // Try to do the request.
    thread::spawn(move || {
        let response = reqwest::blocking::get(url.clone());

        let _ = s.send(match response {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text() {
                        Ok(s) => Ok(s),
                        Err(_) => Err(UrlError::InvalidText(url)),
                    }
                } else {
                    // TODO: allow redirects unless they're circular
                    Err(UrlError::BadStatus(url, response.status()))
                }
            }
            Err(_) => Err(UrlError::ConnectionFailed(url)),
        });
    });

    // Return the request result, or timeout.
    select! {
        recv(r) -> msg => msg.unwrap().map(|u| u.clone()),
        default(Duration::from_secs(TIMEOUT_SECS)) => Err(UrlError::TimedOut(url2.clone()))
    }
}

pub struct FetchedUrls {
    pub urls: Vec<Url>,
    pub malformed_urls: Vec<String>,
}

/// Extract all the URLs from `html_src`.
pub fn fetch_all_urls(html_src: &str, domain: &str) -> FetchedUrls {
    let maybe_urls = parsing::get_urls(&html_src);

    let mut urls = vec![];
    let mut malformed_urls = vec![];

    for maybe_url in maybe_urls.clone() {
        match build_url(domain, &maybe_url) {
            Ok(url) => urls.push(url),
            Err(_) => {
                malformed_urls.push(maybe_url);
            }
        }
    }

    FetchedUrls {
        urls,
        malformed_urls,
    }
}
