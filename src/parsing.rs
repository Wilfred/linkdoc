use scraper::{Html, Selector};

pub fn get_urls(source_str: &str) -> Vec<String> {
    let document = Html::parse_document(source_str);

    let selector = Selector::parse("a").unwrap();

    let mut urls = vec![];
    for node in document.select(&selector) {
        if let Some(url) = node.value().attr("href") {
            urls.push(url.to_owned());
        }
    }

    urls
}
