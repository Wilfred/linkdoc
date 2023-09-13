use clap::{App, Arg};
use colored::*;
use std::io::stdout;
use std::io::Write;
use url::Url;

use crate::fetching::UrlState;

mod crawling;
mod fetching;
mod parsing;

fn main() {
    let matches = App::new("LinkDoctor")
        .version("0.2")
        .about("Walks all the web pages in a domain to find dead links.")
        .author("Wilfred Hughes")
        .arg(Arg::with_name("START URL").required(true))
        .get_matches();

    let start_url_string = matches.value_of("START URL").unwrap();

    // TODO: a proper error message here.
    let start_url = Url::parse(start_url_string).unwrap();

    let domain = start_url
        .domain()
        .expect("I can't find a domain in your URL");

    let mut success_count = 0;
    let mut fail_count = 0;

    for url_state in crawling::crawl(domain, &start_url) {
        match url_state {
            UrlState::Accessible(_) => {
                success_count += 1;
            }
            status => {
                fail_count += 1;
                println!("{}", status);
            }
        }

        print!(
            "{}: {} {}: {}\r",
            "Succeeded".green(),
            success_count,
            "Failed".red(),
            fail_count
        );
        stdout().flush().unwrap();
    }
}
