#![feature(core)]
#![feature(plugin)]
#![plugin(clippy)]

extern crate url;
extern crate html5ever;
extern crate html5ever_dom_sink;

use std::env;
use std::io::stdout;
use std::io::Write;
use url::{Url};

use fetching::UrlState;
use unique::Unique;

mod parsing;
mod fetching;
mod unique;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref start_url_string = args[1];
        let start_url = Url::parse(start_url_string).unwrap();
        
        let html_src = fetching::fetch_url(&start_url);
        let dom = parsing::parse_html(html_src);

        let mut accessible_count = 0;
        let mut error_count = 0;
        
        for path in parsing::get_urls(dom.document).into_iter().unique() {
            // TODO: we should split out the domain and only pass that to url_status
            match fetching::url_status(&start_url, &path) {
                UrlState::Accessible(_) => {
                    accessible_count += 1;
                }
                status @ _ => {
                    error_count += 1;
                    println!("{}", status);
                }
            }

            print!("Accessible: {}, Errors: {}\r", accessible_count, error_count);
            stdout().flush().unwrap();
        }

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
