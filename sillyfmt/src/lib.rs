use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::mem;

pub mod grammar;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Sequence(Vec<Box<Expr>>),
    Container(char, Box<Expr>, char),
    Item(String),
    Delimiter(char),
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
) -> Result<()> {
    let reader = BufReader::new(reader);

    let mut data = String::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        data.push_str(&line);

        if trimmed.is_empty() || format_on_newline {
            do_format(&mut writer, mem::replace(&mut data, String::new()))?;
        }
    }
    if !data.is_empty() {
        do_format(&mut writer, data)?;
    }
    Ok(())
}

fn do_format(mut writer: impl Write, mut data: String) -> Result<()> {
    let (prefix, suffix) = balance_parentheses(&data);
    if let Some(mut prefix) = prefix {
        prefix.push_str(&data);
        data = prefix;
    }
    if let Some(suffix) = suffix {
        data.push_str(&suffix);
    }
    let res = grammar::TopParser::new().parse(&data);

    match res {
        Ok(expr) => {
            let items = format_expr(&*expr);
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
        }
        Err(e) => {
            eprintln!("Couldn't parse data: {:?}", e);
            writeln!(writer, "{}", data)?;
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

fn format_expr(expr: &Expr) -> Vec<R> {
    match *expr {
        Expr::Item(ref s) => {
            let mut out: Vec<R> = s
                .split(|c| c == '\r' || c == '\n')
                .map(|c| {
                    if c.is_empty() {
                        vec![]
                    } else {
                        vec![R::String(c.to_string()), R::Newline]
                    }
                })
                .flatten()
                .collect();
            let _ = out.pop();
            out
        }
        Expr::Sequence(ref s) => {
            let mut out = Vec::new();
            let formatted: Vec<Vec<R>> = s.iter().map(|e| format_expr(e)).collect();
            let (has_breakable, sum) =
                formatted
                    .iter()
                    .fold((false, 0), |(mut breakable, mut sum), it| {
                        for it in it {
                            breakable |= it.is_breakable_delimiter();
                            sum += it.len();
                        }
                        (breakable, sum)
                    });
            if !has_breakable || sum < 32 {
                // It all fits in one line!
                out.extend(
                    formatted
                        .into_iter()
                        .enumerate()
                        .map(|(idx, mut e)| {
                            if e.len() == 1
                                && e.iter().any(|it| it.is_delimiter())
                                && idx != s.len() - 1
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
                            if e.len() == 1 && e.iter().any(|it| it.is_delimiter()) {
                                e.push(R::Newline);
                            }
                            e
                        })
                        .flatten(),
                );
            }
            out
        }
        Expr::Delimiter(c) => vec![R::Delimiter(c, c == ',')],
        Expr::Container(o, ref e, c) => {
            let mut e = format_expr(&e);
            let e_len = e.iter().map(|it| it.len()).sum::<usize>();
            if e_len < 5 && e.iter().all(|e| !e.is_newline()) {
                let mut out = vec![R::Char(o)];
                out.extend(e);
                out.push(R::Char(c));
                out
            } else if e_len < 32 && e.iter().all(|e| !e.is_newline()) {
                let mut out = vec![R::Char(o), R::Space];
                out.extend(e);
                out.push(R::Space);
                out.push(R::Char(c));
                out
            } else {
                let mut out = vec![R::Char(o), R::Indent, R::Newline];
                while let Some(R::Newline) = e.last() {
                    e.pop();
                }
                out.extend(e);
                out.push(R::Unindent);
                out.push(R::Newline);
                out.push(R::Char(c));
                out
            }
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
        ((prefix, suffix))
    } else {
        ((None, None))
    }
}

#[cfg(test)]
mod test {
    use super::{
        do_format,
        balance_parentheses
    };

    #[test]
    fn test_comma_colon_container() {
        let test_str = "{,:}";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "{,:}");
    }
    #[test]
    fn test_colon_container_seq() {
        let test_str = "{:}";
        let mut output = Vec::with_capacity(100);
        do_format(&mut output, test_str.to_string()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "{:}");
    }

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
