extern crate core;
extern crate hyper;
extern crate url;

use std::io::Read;
use self::core::fmt;

use self::hyper::Client;
use self::hyper::status::StatusCode;
use self::url::{Url, UrlParser};

use parsing;

#[derive(Debug,Clone)]
pub enum UrlState {
    Accessible(Url),
    BadStatus(Url, StatusCode),
    ConnectionFailed(Url),
    Malformed(String)
}

impl fmt::Display for UrlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &UrlState::Accessible(ref url) => {
                format!("✔ {}", url).fmt(f)
            }
            &UrlState::BadStatus(ref url, ref status) => {
                format!("✘ {} ({})", url, status).fmt(f)
            }
            &UrlState::ConnectionFailed(ref url) => {
                format!("✘ {} (connection failed)", url).fmt(f)
            }
            &UrlState::Malformed(ref url) => {
                format!("✘ {} (malformed)", url).fmt(f)
            }
        }
    }
}

pub fn url_status(domain: &str, path: &str) -> UrlState {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string).unwrap();

    let mut raw_url_parser = UrlParser::new();
    let url_parser = raw_url_parser.base_url(&base_url);

    return match url_parser.parse(path) {
        Ok(url_value) => {
            let mut client = Client::new();

            let url_string = url_value.serialize();
            let response = client.get(&url_string).send();

            match response {
                Ok(r) => {
                    if let StatusCode::Ok = r.status {
                        UrlState::Accessible(url_value)
                    } else {
                        // TODO: allow redirects unless they're circular
                        UrlState::BadStatus(url_value, r.status)
                    }
                }
                Err(_) => UrlState::ConnectionFailed(url_value)
            }
        },
        Err(_) => UrlState::Malformed(path.to_owned())
    }

}

pub fn fetch_url(url: &Url) -> String {
    // Create a client.
    let mut client = Client::new();

    // Creating an outgoing request.
    let url_string = url.serialize();
    let mut res = client.get(&url_string).send().unwrap();

    // Read the Response.
    let mut body = String::new();
    // FIXME: remove all the .unwrap calls in this function.
    res.read_to_string(&mut body).unwrap();

    body
}

/// Fetch the requested URL, and return a list of all the URLs on the
/// page. We deliberately return strings because we're also interested
/// in malformed URLs.
pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let dom = parsing::parse_html(html_src);

    parsing::get_urls(dom.document)
}
