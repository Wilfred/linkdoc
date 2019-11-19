extern crate hyper;
extern crate url;

use std::fmt;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use self::hyper::status::StatusCode;
use self::hyper::Client;
use self::url::{ParseResult, Url, UrlParser};

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
        match *self {
            UrlState::Accessible(ref url) => format!("✔ {}", url).fmt(f),
            UrlState::BadStatus(ref url, ref status) => format!("✘ {} ({})", url, status).fmt(f),
            UrlState::ConnectionFailed(ref url) => format!("✘ {} (connection failed)", url).fmt(f),
            UrlState::TimedOut(ref url) => format!("✘ {} (timed out)", url).fmt(f),
            UrlState::Malformed(ref url) => format!("✘ {} (malformed)", url).fmt(f),
        }
    }
}

fn build_url(domain: &str, path: &str) -> ParseResult<Url> {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string).unwrap();

    let mut raw_url_parser = UrlParser::new();
    let url_parser = raw_url_parser.base_url(&base_url);

    url_parser.parse(path)
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
                let client = Client::new();

                let url_string = url.serialize();
                let response = client.get(&url_string).send();

                let _ = request_tx.send(match response {
                    Ok(r) => {
                        if let StatusCode::Ok = r.status {
                            UrlState::Accessible(url)
                        } else {
                            // TODO: allow redirects unless they're circular
                            UrlState::BadStatus(url, r.status)
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
    let client = Client::new();

    // Creating an outgoing request.
    let url_string = url.serialize();
    let mut res = client
        .get(&url_string)
        .send()
        .ok()
        .expect("could not fetch URL");

    // Read the Response.
    let mut body = String::new();
    match res.read_to_string(&mut body) {
        // If we can read it as a UTF-8 string, just return that.
        Ok(_) => body,
        // If we can't, it's binary data, so just return an empty string.
        // TODO: It would be cleaner if this function returned bytes.
        // This also assumes that HTML is never in any other encoding.
        Err(_) => String::new(),
    }
}

/// Fetch the requested URL, and return a list of all the URLs on the
/// page. We deliberately return strings because we're also interested
/// in malformed URLs.
pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let dom = parsing::parse_html(&html_src);

    parsing::get_urls(dom.document)
}
