
use std::string::String;
use html5ever::tendril::TendrilSink;

use html5ever::parse_document;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::interface::Attribute;

pub fn parse_html(source_str: &str) -> RcDom {
    parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut source_str.as_bytes())
        .unwrap()
}

pub fn get_urls(handle: Handle) -> Vec<String> {
    let mut urls = vec![];

    let mut anchor_tags = vec![];
    get_elements_by_name(handle, "a", &mut anchor_tags);

    for node in anchor_tags {
        if let NodeData::Element { ref attrs, .. } = node {
            for attr in attrs.borrow().iter() {
                let Attribute {
                    ref name,
                    ref value,
                } = *attr;
                if &*(name.local) == "href" {
                    urls.push(value.to_string());
                }
            }
        }
    }

    urls
}

// Crude tree walker rather than using a full-blown CSS selector library.
fn get_elements_by_name(handle: Handle, element_name: &str, out: &mut Vec<NodeData>) {
    let node = handle;

    if let NodeData::Element {
        ref name,
        ref attrs,
        ref template_contents,
        ..
    } = node.data
    {
        if &*(name.local) == element_name {
            out.push(NodeData::Element {
                name: name.clone(),
                attrs: attrs.clone(),
                template_contents: template_contents.clone(),
                mathml_annotation_xml_integration_point: false,
            });
        }
    }

    for child in node.children.borrow().iter() {
        get_elements_by_name(child.clone(), element_name, out);
    }
}
