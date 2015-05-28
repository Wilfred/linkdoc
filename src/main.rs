#![feature(core)]
extern crate url;
extern crate html5ever;
extern crate html5ever_dom_sink;

use std::env;
use url::{Url};


mod parsing;
mod fetching;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref start_url_string = args[1];
        let start_url = Url::parse(start_url_string).unwrap();
        
        let html_src = fetching::fetch_url(&start_url);
        let dom = parsing::parse_html(html_src);
        
        for path in parsing::get_urls(dom.document) {
            // TODO: we should split out the domain and only pass that to url_status
            println!("{}", fetching::url_status(&start_url, &path));
        }

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
