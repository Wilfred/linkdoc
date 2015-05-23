extern crate hyper;

use std::io::Read;

use self::hyper::Client;
use self::hyper::header::Connection;
use self::hyper::status::StatusCode;

#[derive(Debug)]
pub enum UrlState {
    Accessible,
    BadPath,
    BadDomain
}

pub fn url_status(url: &str) -> UrlState {
    let mut client = Client::new();

    let response = client.get(url).send();

    match response {
        Ok(r) => {
            match r.status {
                StatusCode::Ok => UrlState::Accessible,
                // TODO: allow redirects unless they're circular
                _ => UrlState::BadPath
            }
        }
        Err(_) => UrlState::BadDomain
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
