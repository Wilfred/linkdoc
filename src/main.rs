#![feature(core)]
#![feature(plugin)]

extern crate url;
extern crate html5ever;

use std::env;
use std::io::stdout;
use std::io::Write;
use url::{Url};

use fetching::UrlState;

mod fetching;
mod parsing;
mod crawling;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let start_url_string = &args[1];

        // TODO: a proper error message here.
        let start_url = Url::parse(start_url_string).unwrap();

        let domain = start_url.domain().expect("I can't find a domain in your URL");
        let path_components = start_url.path().expect("I can't find a path in your URL");

        let mut success_count = 0;
        let mut fail_count = 0;

        for url_state in crawling::crawl(&domain, &path_components.join("/")) {
            match url_state {
                UrlState::Accessible(_) => {
                    success_count += 1;
                },
                status => {
                    fail_count += 1;
                    println!("{}", status);
                }
            }

            print!("Succeeded: {} Failed: {}\r", success_count, fail_count);
            stdout().flush().unwrap();
        }

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
