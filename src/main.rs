#![feature(core)]
#![feature(plugin)]
#![plugin(clippy)]

extern crate url;
extern crate html5ever;
extern crate html5ever_dom_sink;

use std::env;
use url::{Url};

use fetching::UrlState;

mod fetching;
mod parsing;
mod crawling;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref start_url_string = args[1];

        // TODO: a proper error message here.
        let start_url = Url::parse(start_url_string).unwrap();

        let domain = start_url.domain().expect("I can't find a domain in your URL");
        let path_components = start_url.path().expect("I can't find a path in your URL");

        for url_state in crawling::crawl(&domain, &path_components.connect("/")) {
            match url_state {
                UrlState::Accessible(_) => (),
                status @ _ => {
                    println!("{}", status);
                }

            }
        }

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
