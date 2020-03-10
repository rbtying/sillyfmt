use sillyfmt::{ParseCursor, ParseNode, ParseTree};
use tree_sitter::{Language, Node, Parser, Tree, TreeCursor};

struct WrappedTree(Tree);
impl ParseTree for WrappedTree {
    fn root_node(&self) -> Box<dyn ParseNode<'_> + '_> {
        Box::new(WrappedNode(self.0.root_node()))
    }

    fn debug_tree(&self) -> String {
        self.0.root_node().to_sexp()
    }
}

struct WrappedNode<'a>(Node<'a>);
impl<'a> ParseNode<'a> for WrappedNode<'a> {
    fn walk(&self) -> Box<dyn ParseCursor<'a> + 'a> {
        Box::new(Cursor(self.0.walk()))
    }
    fn kind(&self) -> String {
        self.0.kind().to_string()
    }
    fn utf8_text(&self, data: &'_ [u8]) -> String {
        self.0.utf8_text(data).unwrap().to_string()
    }
    fn is_named(&self) -> bool {
        self.0.is_named()
    }
}

struct Cursor<'a>(TreeCursor<'a>);
impl<'a> ParseCursor<'a> for Cursor<'a> {
    fn goto_first_child(&mut self) -> bool {
        self.0.goto_first_child()
    }
    fn goto_next_sibling(&mut self) -> bool {
        self.0.goto_next_sibling()
    }
    fn node(&self) -> Box<dyn ParseNode<'a> + 'a> {
        Box::new(WrappedNode(self.0.node()))
    }
    fn field_name(&self) -> Option<String> {
        self.0.field_name().map(|x| x.to_string())
    }
}

extern "C" {
    fn tree_sitter_sillyfmt() -> Language;
}

pub fn parse(s: &str) -> Box<dyn ParseTree> {
    let mut parser = Parser::new();
    parser
        .set_language(unsafe { tree_sitter_sillyfmt() })
        .unwrap();
    let tree = parser.parse(s, None).unwrap();
    Box::new(WrappedTree(tree))
}

#[cfg(test)]
mod tests {
    use std::io::{Result, Write};

    use super::parse;

    fn do_format(writer: impl Write, data: String) -> Result<()> {
        sillyfmt::do_format(writer, data, true, parse)
    }

    #[test]
    fn test_basic_symbol() {
        let test_str = "a=b";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "a = b");
    }

    #[test]
    fn test_basic_string() {
        let test_str = "asdf";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "asdf");
    }

    #[test]
    fn test_sequence() {
        let test_str = "a,b,c,d,e";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "a, b, c, d, e");
    }

    #[test]
    fn test_binop_sequence_in_container() {
        let test_str = "{a:b, c:d, e:f, g:h, i:j, k:l, m:n, o:p, q:r, s:t, u:v, w:x, y:z}";
        let mut output = Vec::with_capacity(1000);
        do_format(&mut output, test_str.to_string()).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap().trim(),
            "{
  a: b,
  c: d,
  e: f,
  g: h,
  i: j,
  k: l,
  m: n,
  o: p,
  q: r,
  s: t,
  u: v,
  w: x,
  y: z
}"
        );
    }

    #[test]
    fn test_labeled_container() {
        let test_str = "struct{a:b, c:d}";
        let mut output = Vec::with_capacity(1000);
        do_format(&mut output, test_str.to_string()).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap().trim(),
            "struct{ a: b, c: d }"
        );
    }

    #[test]
    fn test_comma_colon_container() {
        let test_str = "{,:}";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "{, :}");
    }

    #[test]
    fn test_colon_container_seq() {
        let test_str = "{:}";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "{:}");
    }

    #[test]
    fn test_close_colon() {
        let test_str = "}:";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "{}:");
    }
}
