use scraper::{Html, Selector};
use url::Url;

fn get_urls(source_str: &str) -> Vec<String> {
    let document = Html::parse_document(source_str);

    let mut urls = vec![];

    // Extract URLs from anchor tags.
    let selector = Selector::parse("a").unwrap();
    for node in document.select(&selector) {
        if let Some(url) = node.value().attr("href") {
            urls.push(url.to_owned());
        }
    }

    // Extract URLs from img tags too.
    let selector = Selector::parse("img").unwrap();
    for node in document.select(&selector) {
        if let Some(url) = node.value().attr("src") {
            urls.push(url.to_owned());
        }
    }

    // Also check that CSS links are accessible.
    let selector = Selector::parse("link").unwrap();
    for node in document.select(&selector) {
        if let Some(url) = node.value().attr("href") {
            urls.push(url.to_owned());
        }
    }

    urls
}

fn build_url(domain: &str, path: &str) -> Result<Url, url::ParseError> {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string)?;
    base_url.join(path)
}

pub struct ParsedUrls {
    pub urls: Vec<Url>,
    pub malformed_urls: Vec<String>,
}

/// Extract all the URLs from `html_src`.
pub fn get_parsed_urls(html_src: &str, domain: &str) -> ParsedUrls {
    let maybe_urls = get_urls(html_src);

    let mut urls = vec![];
    let mut malformed_urls = vec![];

    for maybe_url in maybe_urls.clone() {
        match build_url(domain, &maybe_url) {
            Ok(url) => urls.push(url),
            Err(_) => {
                malformed_urls.push(maybe_url);
            }
        }
    }

    ParsedUrls {
        urls,
        malformed_urls,
    }
}
