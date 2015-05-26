extern crate html5ever;
extern crate html5ever_dom_sink;

use std::env;

mod parsing;
mod fetching;
mod crawling;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref url = args[1];
        let html_src = fetching::fetch_url(url);
        let dom = parsing::parse_html(html_src);
        
        for path in parsing::get_urls(dom.document) {
            // TODO: get_urls should return absolute urls.

            // FIXME: this assumes assumes `url` is a domain without a path.
            let mut absolute_url = url.to_string();
            absolute_url = absolute_url + &path;

            println!("URL parsed: {:?}\n", fetching::url_status(&absolute_url));
        }

    } else {
        // TODO: exit non-zero and print proper usage.
        println!("Please provide an URL.")
    }
}
