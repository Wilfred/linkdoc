extern crate url;

use self::url::{Url, ParseResult};

pub fn parse_url(domain: &str, path: &str) -> ParseResult<Url> {
    let mut absolute_url = domain.to_string();
    absolute_url = absolute_url + path;
    Url::parse(&absolute_url)
}
