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

pub fn url_status(url: &Url) -> Result<Url, UrlError> {
    let (s, r) = unbounded();
    let url = url.clone();
    let url2 = url.clone();

    // Try to do the request.
    thread::spawn(move || {
        let response = reqwest::blocking::get(url.clone());

        let _ = s.send(match response {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(url)
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

pub fn fetch_url(url: &Url) -> String {
    // Creating an outgoing request.
    let res = reqwest::blocking::get(url.as_str()).expect("could not fetch URL");

    // Read the body.
    match res.text() {
        Ok(body) => body,
        // TODO: handle malformed data more gracefully.
        Err(_) => String::new(),
    }
}

pub struct FetchedUrls {
    pub urls: Vec<Url>,
    pub malformed_urls: Vec<String>,
}

/// Fetch the requested URL, and all the URLs on the
/// page.
pub fn fetch_all_urls(url: &Url) -> FetchedUrls {
    let html_src = fetch_url(url);
    let maybe_urls = parsing::get_urls(&html_src);

    let mut urls = vec![];
    let mut malformed_urls = vec![];

    if let Some(domain) = url.domain() {
        for maybe_url in maybe_urls.clone() {
            match build_url(domain, &maybe_url) {
                Ok(url) => urls.push(url),
                Err(_) => {
                    malformed_urls.push(maybe_url);
                }
            }
        }
    } else {
        malformed_urls.extend(maybe_urls);
    }

    FetchedUrls {
        urls,
        malformed_urls,
    }
}
