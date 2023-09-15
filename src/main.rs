use clap::{crate_authors, crate_description, crate_version, App, Arg};
use colored::*;
use std::io::stdout;
use std::io::Write;
use url::Url;

mod crawling;
mod fetching;
mod parsing;

#[tokio::main]
async fn my_main() -> Result<(), Box<dyn std::error::Error>> {
    let url_s = "http://www.example.com";
    let start_url = Url::parse(url_s).unwrap();

    Ok(())
}

fn main() {
    let matches = App::new("LinkDoctor")
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
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
            Ok(_) => {
                success_count += 1;
            }
            Err(status) => {
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
