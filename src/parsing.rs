extern crate html5ever;
extern crate tendril;

use std::string::String;
use self::tendril::{ByteTendril, ReadExt};

use html5ever::tokenizer::Attribute;
use html5ever::{parse, one_input};
use html5ever::rcdom::{RcDom, Handle, Element, ElementEnum, NodeEnum};

pub fn parse_html(source_str: String) -> RcDom {
    let mut source = ByteTendril::new();
    source_str.as_bytes().read_to_tendril(&mut source).unwrap();
    let source = source.try_reinterpret().unwrap();

    parse(one_input(source), Default::default())
}

pub fn get_urls(handle: Handle) -> Vec<String> {
    let mut urls = vec![];

    let mut anchor_tags = vec![];
    get_elements_by_name(handle, "a", &mut anchor_tags);

    for node in anchor_tags {
        if let Element(_, _, ref attrs) = node {
            for attr in attrs.iter() {
                let Attribute { ref name, ref value } = *attr;
                if name.local.as_slice() == "href" {
                    urls.push(value.to_string());
                }
            }
        }
    }

    urls
}

// Crude tree walker rather than using a full-blown CSS selector library.
fn get_elements_by_name(handle: Handle, element_name: &str, out: &mut Vec<NodeEnum>) {
    let node = handle.borrow();

    if let Element(ref name, _, ref attrs) = node.node {
        if name.local.as_slice() == element_name {
            out.push(Element(name.clone(), ElementEnum::Normal, attrs.clone()));
        }
    }

    for child in &node.children {
        get_elements_by_name(child.clone(), element_name, out);
    }
}
