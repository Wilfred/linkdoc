extern crate html5ever;
extern crate html5ever_dom_sink;

use std::string::String;

use html5ever::tokenizer::Attribute;

use self::html5ever_dom_sink::common::{NodeEnum, Element};
use self::html5ever_dom_sink::rcdom::Handle;

pub fn get_urls(handle: Handle) -> Vec<String> {
    let mut urls = Vec::new();
    
    let mut anchor_tags = Vec::new();
    get_elements_by_name(handle, "a", &mut anchor_tags);

    for node in anchor_tags {
        match node {
            Element(_, ref attrs) => {
                for attr in attrs.iter() {
                    let Attribute { ref name, ref value } = *attr;
                    if name.local.as_slice() == "href" {
                        urls.push(value.to_string());
                    }
                }
            }
            _ => ()
        }
    }

    urls
}

// Crude tree walker rather than using a full-blown CSS selector library.
fn get_elements_by_name(handle: Handle, element_name: &str, out: &mut Vec<NodeEnum>) {
    let node = handle.borrow();

    match node.node {
        Element(ref name, ref attrs) => {
            if name.local.as_slice() == element_name {
                out.push(Element(name.clone(), attrs.clone()));
            }
        }
        _ => ()
    };

    for child in node.children.iter() {
        get_elements_by_name(child.clone(), element_name, out);
    }
}

