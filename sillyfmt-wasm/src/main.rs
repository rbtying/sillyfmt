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

fn main() {
    stdweb::initialize();

    let input: TextAreaElement = document()
        .query_selector("#text-input")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    let output = document().query_selector("#text-output").unwrap().unwrap();

    input.add_event_listener(enclose!( (input, output) move |_: InputEvent| {
        let s = input.value();
        let mut out = Vec::new();
        let _ = silly_format(Cursor::new(s), Cursor::new(&mut out), false, |x| {
            Box::new(WrappedTree(js!(
                return parser.parse(@{x});
            )))
        });
        let formatted = String::from_utf8_lossy(&out);
        output.set_text_content(&*formatted);
    }));

    stdweb::event_loop();
}
