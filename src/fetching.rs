use colored::*;
use crossbeam_channel::{select, unbounded};
use reqwest::StatusCode;
use std::fmt;
use std::thread;
use std::time::Duration;
use url::Url;

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
