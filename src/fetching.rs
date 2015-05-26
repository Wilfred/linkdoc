extern crate hyper;
extern crate url;

use std::io::Read;

use self::hyper::Client;
use self::hyper::header::Connection;
use self::hyper::status::StatusCode;
use self::url::{Url};

#[derive(Debug)]
pub enum UrlState {
    Accessible(Url),
    BadPath(Url),
    BadDomain(Url),
    Malformed(String)
}

pub fn url_status(url: &str) -> UrlState {
    let mut client = Client::new();

    return match Url::parse(url) {
        Ok(url_value) => {
            let response = client.get(url).send();

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
        Err(_) => UrlState::Malformed(url.to_string())
    }

}

pub fn fetch_url(url: &str) -> String {
    // Create a client.
    let mut client = Client::new();

    // Creating an outgoing request.
    let mut res = client.get(url)
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
