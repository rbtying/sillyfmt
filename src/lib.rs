use std::io::{BufRead, BufReader, Read, Result, Write};

pub fn silly_format(reader: impl Read, mut writer: impl Write) -> Result<()> {
    let reader = BufReader::new(reader);

    for line in reader.lines() {
        let line = line?;
        format_line(&line, &mut writer)?;
        write!(writer, "\n")?
    }
    Ok(())
}

fn format_line(line: &str, mut writer: impl Write) -> Result<()> {
    let mut sequence = vec![];

    #[derive(Clone, Copy)]
    enum C {
        Whitespace,
        Alphanum,
        Symbol,
    }
    let mut start = Some(0);
    let mut last_c_type = C::Whitespace;

    for (idx, c) in line.char_indices() {
        let c_type = if c.is_whitespace() {
            C::Whitespace
        } else if c.is_alphanumeric()
            || c == '.'
            || c == '_'
            || c == '-'
            || c == '/'
            || c == '\\'
            || c == ':'
            || c == '\''
            || c == '\"'
        {
            C::Alphanum
        } else {
            C::Symbol
        };

        match (last_c_type, c_type) {
            (C::Whitespace, C::Whitespace) | (C::Alphanum, C::Alphanum) => (),

            (C::Whitespace, C::Alphanum) | (C::Whitespace, C::Symbol) => {
                start = Some(idx);
            }
            (C::Symbol, C::Alphanum) | (C::Symbol, C::Symbol) | (C::Alphanum, C::Symbol) => {
                sequence.push(&line[start.unwrap()..idx]);
                start = Some(idx);
            }

            (C::Alphanum, C::Whitespace) | (C::Symbol, C::Whitespace) => {
                sequence.push(&line[start.unwrap()..idx]);
                start = None;
            }
        }
        last_c_type = c_type;
    }
    if let Some(start) = start {
        if start < line.len() {
            sequence.push(&line[start..]);
        }
    }

    let mut indent = 0;

    macro_rules! newline {
        () => {{
            write!(writer, "\n")?;
            for _ in 0..indent {
                write!(writer, "  ")?;
            }
        }};
    }
    let mut oneline = false;

    for (idx, token) in sequence.iter().enumerate() {
        match *token {
            "}" | ")" | "]" => {
                if !oneline {
                    indent -= 1;
                    newline!();
                }
                oneline = false;
            }
            _ => (),
        }

        write!(writer, "{} ", token)?;

        match *token {
            o @ "{" | o @ "(" | o @ "[" => {
                if !oneline {
                    let mut offset = 1;
                    oneline = loop {
                        let lookahead = sequence.get(idx + offset).map(|c| *c);
                        match (o, lookahead) {
                            ("{", Some("}")) | ("(", Some(")")) | ("[", Some("]")) => break true,
                            (_, None) => break false,
                            _ => (),
                        }
                        offset += 1;
                        if offset > 5 {
                            break false;
                        }
                    };
                }
                if !oneline {
                    indent += 1;
                    newline!();
                }
            }
            "," => {
                if !oneline {
                    newline!();
                }
            }
            _ => (),
        }
    }

    write!(writer, "\n")
}
