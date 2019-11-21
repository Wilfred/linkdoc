use scraper::{Html, Selector};

pub fn get_urls(source_str: &str) -> Vec<String> {
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
