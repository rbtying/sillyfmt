use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::mem;

pub trait ParseTree {
    fn root_node(&self) -> Box<dyn ParseNode<'_> + '_>;
}

pub trait ParseCursor<'a> {
    fn goto_first_child(&mut self) -> bool;
    fn goto_next_sibling(&mut self) -> bool;
    fn node(&self) -> Box<dyn ParseNode<'a> + 'a>;
    fn field_name(&self) -> Option<String>;
}

pub trait ParseNode<'a> {
    fn walk(&self) -> Box<dyn ParseCursor<'a> + 'a>;
    fn kind(&self) -> String;
    fn utf8_text(&self, data: &'_ [u8]) -> String;
    fn is_named(&self) -> bool;
}

fn swap_char(c: char) -> char {
    match c {
        '}' => '{',
        '{' => '}',
        ']' => '[',
        '[' => ']',
        '(' => ')',
        ')' => '(',
        _ => unreachable!(),
    }
}

pub fn silly_format(
    reader: impl Read,
    mut writer: impl Write,
    format_on_newline: bool,
    parser: impl Fn(&str) -> Box<dyn ParseTree>,
) -> Result<()> {
    let reader = BufReader::new(reader);

    let mut data = String::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        data.push_str(&line);

        if trimmed.is_empty() || format_on_newline {
            do_format(&mut writer, mem::replace(&mut data, String::new()), &parser)?;
        }
    }
    if !data.is_empty() {
        do_format(&mut writer, data, &parser)?;
    }
    Ok(())
}

fn format_parse_cursor<'a, 'b>(
    mut cursor: Box<dyn ParseCursor<'a> + 'a>,
    data: &'b [u8],
) -> Vec<R> {
    let mut out = Vec::new();
    let node = cursor.node();
    match node.kind().as_str() {
        "symbol" => {
            let symbol = node.utf8_text(data);
            out.push(if symbol.chars().count() == 1 {
                R::Delimiter(symbol.chars().next().unwrap(), false)
            } else {
                R::String(symbol)
            });
        }
        "binary_op" => {
            let mut formatted = vec![];
            if cursor.goto_first_child() {
                // Try to format all the children.
                loop {
                    let inner_node = cursor.node();
                    if inner_node.kind() == "symbol" {
                        let symbol = inner_node.utf8_text(data);
                        let symbol_r = if symbol.chars().count() == 1 {
                            R::Delimiter(symbol.chars().next().unwrap(), false)
                        } else {
                            R::String(symbol)
                        };
                        if let R::Delimiter(':', _) = symbol_r {
                            formatted.push(vec![symbol_r, R::Space]);
                        } else {
                            formatted.push(vec![R::Space, symbol_r, R::Space]);
                        }
                    } else {
                        formatted.push(format_parse_cursor(inner_node.walk(), data));
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            format_seq(formatted, &mut out);
        }
        "ERROR" | "text" | "time" => out.push(R::String(node.utf8_text(data))),
        "," => out.push(R::Delimiter(',', true)),
        "container" => {
            let mut formatted_children = vec![];
            let mut open = ' ';
            let mut close = ' ';
            if cursor.goto_first_child() {
                // Try to format all the children.
                loop {
                    match cursor.field_name().as_ref().map(|s| &s[..]) {
                        Some("open") => {
                            if let Some(c) = cursor.node().utf8_text(data).chars().next() {
                                open = c;
                            }
                        }
                        Some("close") => {
                            if let Some(c) = cursor.node().utf8_text(data).chars().next() {
                                close = c;
                            }
                        }
                        _ => {
                            formatted_children.push(format_parse_cursor(cursor.node().walk(), data))
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            let mut e = vec![];
            format_seq(formatted_children, &mut e);
            let e_len = e.iter().map(|it| it.len()).sum::<usize>();
            if e_len < 5 && e.iter().all(|e| !e.is_newline()) {
                out.push(R::Char(open));
                out.extend(e);
                out.push(R::Char(close));
            } else if e_len < 32 && e.iter().all(|e| !e.is_newline()) {
                out.push(R::Char(open));
                out.push(R::Space);
                out.extend(e);
                out.push(R::Space);
                out.push(R::Char(close));
            } else {
                out.push(R::Char(open));
                out.push(R::Indent);
                out.push(R::Newline);
                while let Some(R::Newline) = e.last() {
                    e.pop();
                }
                out.extend(e);
                out.push(R::Unindent);
                out.push(R::Newline);
                out.push(R::Char(close));
            }
        }
        _ if node.is_named() => {
            let mut formatted = vec![];
            if cursor.goto_first_child() {
                // Try to format all the children.
                loop {
                    formatted.push(format_parse_cursor(cursor.node().walk(), data));
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            format_seq(formatted, &mut out);
        }
        _ => {
            out.push(R::String(node.utf8_text(data)));
        }
    }

    out
}

fn format_seq(formatted: Vec<Vec<R>>, out: &mut Vec<R>) {
    let (has_breakable, sum) = formatted
        .iter()
        .fold((false, 0), |(mut breakable, mut sum), it| {
            for it in it {
                breakable |= it.is_breakable_delimiter();
                sum += it.len();
            }
            (breakable, sum)
        });
    if !has_breakable || sum < 32 {
        let last = if formatted.is_empty() {
            0
        } else {
            formatted.len() - 1
        };
        // It all fits in one line!
        out.extend(
            formatted
                .into_iter()
                .enumerate()
                .map(|(idx, mut e)| {
                    if e.len() == 1 && e.iter().any(|it| it.is_delimiter()) && idx != last {
                        e.push(R::Space);
                    }
                    e
                })
                .flatten(),
        );
    } else {
        // Add newlines after delimiters
        out.extend(
            formatted
                .into_iter()
                .map(|mut e| {
                    if e.len() == 1 && e.iter().any(|it| it.is_delimiter()) {
                        e.push(R::Newline);
                    }
                    e
                })
                .flatten(),
        );
    }
}

pub fn do_format(
    mut writer: impl Write,
    mut data: String,
    parser: impl Fn(&str) -> Box<dyn ParseTree>,
) -> Result<()> {
    let (prefix, suffix) = balance_parentheses(&data);
    if let Some(mut prefix) = prefix {
        prefix.push_str(&data);
        data = prefix;
    }
    if let Some(suffix) = suffix {
        data.push_str(&suffix);
    }

    let tree = parser(&data);
    let items = format_parse_cursor(tree.root_node().walk(), data.as_bytes());
    let mut indent = 0;

    for item in items.iter() {
        match item {
            R::String(s) => write!(writer, "{}", s)?,
            R::Char(c) | R::Delimiter(c, _) => write!(writer, "{}", c)?,
            R::Space => write!(writer, " ")?,
            R::Indent => {
                indent += 1;
            }
            R::Unindent => {
                indent -= 1;
            }
            R::Newline => {
                write!(writer, "\n")?;
                for _ in 0..indent {
                    write!(writer, "  ")?;
                }
            }
        }
    }

    write!(writer, "\n")?;
    Ok(())
}

#[derive(Debug)]
enum R {
    String(String),
    Delimiter(char, bool),
    Char(char),
    Space,
    Newline,
    Indent,
    Unindent,
}

impl R {
    fn len(&self) -> usize {
        match self {
            R::String(s) => s.len(),
            _ => 1,
        }
    }

    fn is_newline(&self) -> bool {
        match self {
            R::Newline => true,
            _ => false,
        }
    }

    fn is_breakable_delimiter(&self) -> bool {
        match self {
            R::Delimiter(_, breakable) => *breakable,
            _ => false,
        }
    }

    fn is_delimiter(&self) -> bool {
        match self {
            R::Delimiter(_, _) => true,
            _ => false,
        }
    }
}

fn balance_parentheses(s: &'_ str) -> (Option<String>, Option<String>) {
    let mut stk_map: HashMap<char, usize> = HashMap::new();
    // Close all opened parens
    for c in s.chars() {
        match c {
            c @ '{' | c @ '[' | c @ '(' => {
                // Mark the number of opens
                *stk_map.entry(c).or_insert(0) += 1;
            }
            c @ '}' | c @ ']' | c @ ')' => {
                let o = swap_char(c);
                let o_e = stk_map.entry(o).or_insert(0);
                if *o_e > 0 {
                    *o_e -= 1;
                } else {
                    // This is a close without a corresponding open.
                    // Track it separately.
                    *stk_map.entry(c).or_insert(0) += 1;
                }
            }
            _ => (),
        }
    }

    if stk_map.values().any(|ct| *ct > 0) {
        let mut prefix = None;
        let mut suffix = None;
        for (c, ct) in stk_map.into_iter() {
            if ct == 0 {
                continue;
            }
            match c {
                c @ '{' | c @ '[' | c @ '(' => {
                    if suffix.is_none() {
                        suffix = Some(String::new());
                    }
                    let suffix = suffix.as_mut().expect("must be set");
                    for _ in 0..ct {
                        suffix.push(swap_char(c));
                    }
                }
                c @ '}' | c @ ']' | c @ ')' => {
                    if prefix.is_none() {
                        prefix = Some(String::new());
                    }
                    let prefix = prefix.as_mut().expect("must be set");
                    for _ in 0..ct {
                        prefix.push(swap_char(c));
                    }
                }
                _ => unreachable!(),
            }
        }
        (prefix, suffix)
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod test {
    use super::balance_parentheses;

    #[test]
    fn test_basic_hanging_open() {
        let (p, s) = balance_parentheses("((");
        assert_eq!(p, None);
        assert_eq!(s, Some("))".to_owned()));
    }

    #[test]
    fn test_basic_hanging_close() {
        let (p, s) = balance_parentheses("))");
        assert_eq!(p, Some("((".to_owned()));
        assert_eq!(s, None);
    }

    #[test]
    fn test_backward_pair() {
        let (p, s) = balance_parentheses(")(");
        assert_eq!(p, Some("(".to_owned()));
        assert_eq!(s, Some(")".to_owned()));
    }
}
