extern crate hyper;

use hyper::Client;
use hyper::status::StatusCode;

use std::env;

#[derive(Debug)]
enum UrlState {
    Accessible,
    BadPath,
    BadDomain
}

fn url_accessible(url: &str) -> UrlState {
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

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref url = args[1];
        println!("url status: {:?}", url_accessible(url));

    } else {
        // TODO: exit non-zero.
        println!("Please provide an URL.")
    }
}
