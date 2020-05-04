use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::mem;

pub trait ParseTree {
    fn root_node(&self) -> Box<dyn ParseNode<'_> + '_>;
    fn debug_tree(&self) -> String;
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
    fn start_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
    fn utf8_text(&self, data: &'_ [u8]) -> String;
    fn is_named(&self) -> bool;
}

pub fn silly_format(
    reader: impl Read,
    mut writer: impl Write,
    format_on_newline: bool,
    mut print_debug: Option<impl Write>,
    parser: impl Fn(String) -> (Box<dyn ParseTree>, String),
) -> Result<()> {
    let reader = BufReader::new(reader);

    let mut data = String::new();

    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() {
            data.push_str(&line);
            data.push('\n');
        }

        if line.is_empty() || format_on_newline {
            do_format(
                &mut writer,
                mem::replace(&mut data, String::new()),
                print_debug.as_mut(),
                &parser,
            )?;
        }
    }
    if !data.is_empty() {
        do_format(&mut writer, data, print_debug.as_mut(), &parser)?;
    }
    Ok(())
}

fn format_parse_cursor<'a, 'b, DW: Write>(
    mut cursor: Box<dyn ParseCursor<'a> + 'a>,
    data: &'b [u8],
    from: usize,
    to: usize,
    mut print_debug: Option<DW>,
) -> (Vec<R>, Option<DW>) {
    let mut out = Vec::new();
    let node = cursor.node();
    if let Ok(p) = std::str::from_utf8(&data[from..node.start_byte()]) {
        out.extend(minimize_whitespace(p));
    }
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
                    if inner_node.kind() == "symbol" || inner_node.kind() == "conflicting_symbol" {
                        let symbol = inner_node.utf8_text(data);
                        let symbol_r = if symbol.chars().count() == 1 {
                            R::Delimiter(symbol.chars().next().unwrap(), false)
                        } else {
                            R::String(symbol)
                        };
                        formatted.push(match symbol_r {
                            R::Delimiter(':', _) => vec![symbol_r],
                            _ => vec![R::Space, symbol_r],
                        });
                    } else {
                        let (mut res, print_debug_) = format_parse_cursor(
                            inner_node.walk(),
                            data,
                            inner_node.start_byte(),
                            inner_node.end_byte(),
                            print_debug,
                        );
                        print_debug = print_debug_;
                        if inner_node.kind() == "subbinary_op" {
                            res.remove(0);
                        } else {
                            match res.first() {
                                Some(R::Space) | Some(R::Newline) => (),
                                _ => formatted.push(vec![R::Space]),
                            }
                        }
                        formatted.push(res);
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            format_seq(formatted, &mut out);
        }
        "text" | "time" => out.extend(minimize_whitespace(&node.utf8_text(data))),
        "," => out.push(R::Delimiter(',', true)),
        "container" => {
            let mut formatted_children = vec![];
            let mut open = ' ';
            let mut close = ' ';
            if cursor.goto_first_child() {
                // Try to format all the children.
                let mut seq = node.start_byte();
                loop {
                    let node = cursor.node();
                    let end_byte = node.end_byte();
                    match cursor.field_name().as_ref().map(|s| &s[..]) {
                        Some("open") => {
                            if let Some(c) = node.utf8_text(data).chars().next() {
                                open = c;
                            }
                        }
                        Some("close") => {
                            if let Some(c) = node.utf8_text(data).chars().next() {
                                close = c;
                            }
                        }
                        _ => {
                            let (res, print_debug_) =
                                format_parse_cursor(node.walk(), data, seq, end_byte, print_debug);
                            formatted_children.push(res);
                            print_debug = print_debug_;
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                    seq = end_byte;
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
                loop {
                    match e.last() {
                        Some(R::Newline) | Some(R::Space) => {
                            e.pop();
                        }
                        _ => break,
                    }
                }
                out.extend(e);
                out.push(R::Unindent);
                out.push(R::Newline);
                out.push(R::Char(close));
            }
        }
        "comma_delimited_sequence" => {
            let mut formatted = vec![];
            if cursor.goto_first_child() {
                // Try to format all the children.
                loop {
                    let node = cursor.node();
                    let (res, print_debug_) = format_parse_cursor(
                        node.walk(),
                        data,
                        node.start_byte(),
                        node.end_byte(),
                        print_debug,
                    );
                    formatted.push(res);
                    print_debug = print_debug_;
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            format_seq(formatted, &mut out);
        }
        _ if node.is_named() => {
            let mut formatted = vec![];
            let mut seq = node.start_byte();
            if cursor.goto_first_child() {
                // Try to format all the children.
                loop {
                    let node = cursor.node();
                    let end = node.end_byte();
                    let (res, print_debug_) =
                        format_parse_cursor(node.walk(), data, seq, end, print_debug);
                    formatted.push(res);
                    print_debug = print_debug_;
                    seq = end;
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            format_seq(formatted, &mut out);
        }
        _ => {
            out.extend(minimize_whitespace(&node.utf8_text(data)));
        }
    }

    if let Ok(s) = std::str::from_utf8(&data[node.end_byte()..to]) {
        out.extend(minimize_whitespace(s));
    }
    (out, print_debug)
}

fn minimize_whitespace(s: &'_ str) -> Vec<R> {
    match s.len() {
        0 => vec![],
        1 => vec![match s.chars().next().unwrap() {
            ' ' => R::Space,
            '\n' => R::Newline,
            c => R::Char(c),
        }],
        _ => {
            let mut out = vec![];

            let mut s_out = String::new();
            let mut s_whitespace = None;

            for c in s.trim().chars() {
                if c.is_whitespace() {
                    out.push(R::String(mem::replace(&mut s_out, String::new())));
                    s_whitespace = Some(match (s_whitespace, c) {
                        (Some(_), c) if c == '\n' => c,
                        (Some(w), _) => w,
                        (None, c) => c,
                    });
                } else {
                    match s_whitespace {
                        Some(' ') => out.push(R::Space),
                        Some('\n') => out.push(R::Newline),
                        Some(w) => out.push(R::Char(w)),
                        _ => (),
                    }
                    s_whitespace = None;
                    s_out.push(c);
                }
            }
            match s_whitespace {
                Some(' ') => out.push(R::Space),
                Some('\n') => out.push(R::Newline),
                Some(w) => out.push(R::Char(w)),
                _ => (),
            }
            if !s_out.is_empty() {
                out.push(R::String(s_out));
            }
            out
        }
    }
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
                    if e.len() == 1 && e.iter().any(|it| it.is_breakable_delimiter()) && idx != last
                    {
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
                    if e.len() == 1 && e.iter().any(|it| it.is_breakable_delimiter()) {
                        e.push(R::Newline);
                    }
                    e
                })
                .flatten(),
        );
    }
}

pub fn do_format(
    writer: impl Write,
    data: String,
    mut print_debug: Option<impl Write>,
    parser: impl Fn(String) -> (Box<dyn ParseTree>, String),
) -> Result<()> {
    let (tree, data) = parser(data);
    if let Some(debug) = print_debug.as_mut() {
        writeln!(debug, "==============================")?;
        writeln!(debug, "{}", tree.debug_tree())?;
    }

    let data_as_bytes = data.as_bytes();
    let (items, mut print_debug) = format_parse_cursor(
        tree.root_node().walk(),
        data_as_bytes,
        0,
        data_as_bytes.len(),
        print_debug,
    );
    if let Some(mut debug) = print_debug.as_mut() {
        writeln!(debug, "------------------------------")?;
        write_output(items.iter(), &mut debug)?;
        writeln!(debug, "==============================")?;
    }

    write_output(items.iter(), writer)?;

    Ok(())
}

fn write_output<'a>(items: impl IntoIterator<Item = &'a R>, mut writer: impl Write) -> Result<()> {
    let mut indent = 0;
    for item in items {
        match item {
            R::String(s) => {
                write!(writer, "{}", s)?;
            }
            R::Char(c) | R::Delimiter(c, _) => {
                write!(writer, "{}", c)?;
            }
            R::Space => {
                write!(writer, " ")?;
            }
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
}
