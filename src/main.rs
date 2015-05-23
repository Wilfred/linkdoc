extern crate html5ever;
extern crate html5ever_dom_sink;

use std::env;

mod parsing;
mod fetching;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref url = args[1];
        let html_src = fetching::fetch_url(url);
        let dom = parsing::parse_html(html_src);
        
        for url in parsing::get_urls(dom.document) {
            println!("URL: {}", url);
        }

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
