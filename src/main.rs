extern crate html5ever;
extern crate html5ever_dom_sink;

use std::io::{self, Read};
use std::default::Default;
use std::string::String;

use html5ever::{parse, one_input};
use html5ever_dom_sink::common::{NodeEnum, Element};
use html5ever_dom_sink::rcdom::{RcDom, Handle};

fn print_urls(handle: Handle) {
    let node = handle.borrow();

    match node.node {
        Element(ref name, ref attrs) => {
            print!("<{}", name.local);
            for attr in attrs.iter() {
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        }
        _ => ()
    }

    for child in node.children.iter() {
        print_urls(child.clone());
    }
}

// Crude tree walker rather than using a full-blown CSS selector library.
fn get_elements_by_name(handle: Handle, name: String) -> Vec<NodeEnum> {
    let mut elements = Vec::new();

    elements
}

fn parse_html(source: String) -> RcDom {
    parse(one_input(source), Default::default())
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let dom = parse_html(input);
    print_urls(dom.document);
}
