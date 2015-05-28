extern crate core;
extern crate hyper;
extern crate url;

use std::io::Read;
use self::core::fmt;

use self::hyper::Client;
use self::hyper::header::Connection;
use self::hyper::status::StatusCode;
use self::url::{Url, UrlParser};

#[derive(Debug)]
pub enum UrlState {
    Accessible(Url),
    BadPath(Url),
    BadDomain(Url),
    Malformed(String)
}

impl fmt::Display for UrlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &UrlState::Accessible(ref url) => {
                format!("✔ {}", url).fmt(f)
            }
            &UrlState::BadPath(ref url) => {
                format!("✘ {} (404)", url).fmt(f)
            }
            &UrlState::BadDomain(ref url) => {
                format!("✘ {} (no domain)", url).fmt(f)
            }
            &UrlState::Malformed(ref url) => {
                format!("✘ {} (malformed)", url).fmt(f)
            }
        }
    }
}

pub fn url_status(base_url: &Url, path: &str) -> UrlState {
    let mut client = Client::new();

    let mut raw_url_parser = UrlParser::new();
    let url_parser = raw_url_parser.base_url(base_url);

    return match url_parser.parse(path) {
        Ok(url_value) => {
            let url_string = url_value.serialize();
            let response = client.get(&url_string).send();

            match response {
                Ok(r) => {
                    match r.status {
                        StatusCode::Ok => UrlState::Accessible(url_value),
                        // TODO: allow redirects unless they're circular
                        _ => UrlState::BadPath(url_value)
                    }
                }
                Err(_) => UrlState::BadDomain(url_value)
            }
        },
        Err(_) => UrlState::Malformed(path.to_string())
    }

}

pub fn fetch_url(url: &Url) -> String {
    // Create a client.
    let mut client = Client::new();

    // Creating an outgoing request.
    let url_string = url.serialize();
    let mut res = client.get(&url_string)
        // set a header
        .header(Connection::close())
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    // FIXME: remove all the .unwrap calls in this function.
    res.read_to_string(&mut body).unwrap();

    body
}
