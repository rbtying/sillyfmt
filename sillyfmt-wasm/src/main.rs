#[macro_use]
extern crate stdweb;

use std::io::Cursor;

use serde::{Deserialize, Serialize};
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::document;
use stdweb::web::event::InputEvent;
use stdweb::web::html_element::TextAreaElement;

use sillyfmt::{silly_format, ParseCursor, ParseNode, ParseTree};

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrappedTree(stdweb::Value);
impl ParseTree for WrappedTree {
    fn root_node(&self) -> Box<dyn ParseNode<'_> + '_> {
        Box::new(WrappedNode(js!(
            return @{&self.0}.rootNode;
        )))
    }
    fn debug_tree(&self) -> String {
        js!(
            return @{&self.0}.rootNode.toString();
        )
        .try_into()
        .unwrap()
    }
}
js_serializable!(WrappedTree);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrappedNode(stdweb::Value);
impl<'a> ParseNode<'a> for WrappedNode {
    fn walk(&self) -> Box<dyn ParseCursor<'a> + 'a> {
        Box::new(WrappedCursor(js!(
            return @{&self.0}.walk();
        )))
    }
    fn kind(&self) -> String {
        js!(
            return @{&self.0}.type;
        )
        .try_into()
        .unwrap()
    }
    fn start_byte(&self) -> usize {
        js!(
            return @{&self.0}.startIndex;
        )
        .try_into()
        .unwrap()
    }
    fn end_byte(&self) -> usize {
        js!(
            return @{&self.0}.endIndex;
        )
        .try_into()
        .unwrap()
    }
    fn utf8_text(&self, _: &'_ [u8]) -> String {
        js!(
            return @{&self.0}.text;
        )
        .try_into()
        .unwrap()
    }
    fn is_named(&self) -> bool {
        js!(
            return @{&self.0}.isNamed();
        )
        .try_into()
        .unwrap()
    }
}
js_serializable!(WrappedNode);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrappedCursor(stdweb::Value);
impl<'a> ParseCursor<'a> for WrappedCursor {
    fn goto_first_child(&mut self) -> bool {
        js!(
            return @{&self.0}.gotoFirstChild();
        )
        .try_into()
        .unwrap()
    }
    fn goto_next_sibling(&mut self) -> bool {
        js!(
            return @{&self.0}.gotoNextSibling();
        )
        .try_into()
        .unwrap()
    }
    fn node(&self) -> Box<dyn ParseNode<'a> + 'a> {
        Box::new(WrappedNode(js!(
            return @{&self.0}.currentNode();
        )))
    }
    fn field_name(&self) -> Option<String> {
        js!(
            return @{&self.0}.currentFieldName();
        )
        .try_into()
        .unwrap()
    }
}
js_serializable!(WrappedCursor);

fn run_format(s: String, debug: bool) -> String {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let _ = silly_format(
        Cursor::new(s),
        Cursor::new(&mut out),
        false,
        if debug {
            Some(Cursor::new(&mut err))
        } else {
            None
        },
        |x| {
            let tree = js!( return parser.parse(@{x}); );
            let tree_str: String = js!( return @{&tree}.rootNode.text ).try_into().unwrap();
            (Box::new(WrappedTree(tree)), tree_str)
        },
    );
    let formatted = String::from_utf8_lossy(&out);
    if !err.is_empty() {
        let formatted_err = &*String::from_utf8_lossy(&err);
        js!( console.log(@{formatted_err}); );
    }
    formatted.to_string()
}

fn main() {
    stdweb::initialize();

    let debug: bool = js!(
        const url = new URL(location);
        const params = url.searchParams;
        const debug = params.get("debug") != null;
        return debug;
    )
    .try_into()
    .unwrap();

    let query_str: String = js!(
        try {
            if (location.hash.startsWith("#b64:")) {
                return decodeURIComponent(escape(atob(decodeURI(location.hash.substring(5)))));
            } else {
                return decodeURI(location.hash.substring(1));
            }
        } catch (err) {
            console.log(err);
            location.hash = "";
            return "";
        }
    )
    .try_into()
    .unwrap();

    let input: TextAreaElement = document()
        .query_selector("#text-input")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    input.set_value(&query_str);
    let output = document().query_selector("#text-output").unwrap().unwrap();
    let formatted = run_format(query_str, debug);
    output.set_text_content(&formatted);

    input.add_event_listener(enclose!( (input, output) move |_: InputEvent| {
        let s = input.value();
        js!(
            const jsstr = @{&s};
            if (jsstr.length >= 64 || jsstr.startsWith("b64:")) {
                location.hash = "b64:" + encodeURI(btoa(unescape(encodeURIComponent(jsstr))));
            } else if (jsstr.length <= 1900) {
                location.hash = encodeURI(jsstr);
            } else {
                location.hash = "";
            }
        );
        let formatted = run_format(s, debug);
        output.set_text_content(&formatted);
    }));

    stdweb::event_loop();
}
