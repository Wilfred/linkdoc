#![feature(core)]
#![feature(plugin)]
#![plugin(clippy)]

extern crate url;
extern crate html5ever;
extern crate html5ever_dom_sink;

use std::env;
use url::{Url};

use fetching::UrlState;

mod parsing;
mod fetching;
mod unique;
mod crawling;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref start_url_string = args[1];
        let start_url = Url::parse(start_url_string).unwrap();

        let results = crawling::crawl(start_url.domain().unwrap(), "/");
        let bad_results: Vec<_> = results.into_iter().filter(|state| match state {
            &UrlState::Accessible(..) => false,
            _ => true
        }).collect();
        println!("failed: {:?}", bad_results);

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
