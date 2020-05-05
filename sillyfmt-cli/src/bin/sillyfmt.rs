use atty::Stream;
use std::env;
use std::io;

use rustyline::{error::ReadlineError, Editor};
use sillyfmt::silly_format_iter;
use sillyfmt_tree_sitter::parse;

fn main() -> io::Result<()> {
    let mut format_on_newline = false;
    let mut print_debug = false;
    for arg in env::args() {
        if arg == "--newline" {
            format_on_newline = true;
        }
        if arg == "--debug" {
            print_debug = true;
        }
    }
    if atty::is(Stream::Stdin) && !format_on_newline {
        println!("Hit enter twice to format, or re-run with --newline");
    }
    let rl = Editor::<()>::new();
    struct EditorIter {
        editor: Editor<()>,
    }

    impl Iterator for EditorIter {
        type Item = io::Result<String>;
        fn next(&mut self) -> Option<io::Result<String>> {
            match self.editor.readline("") {
                Ok(line) => Some(Ok(line)),
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => None,
                Err(ReadlineError::Io(e)) => Some(Err(e)),
                Err(e) => {
                    eprintln!("Unexpected err {:?}", e);
                    Some(Err(io::Error::new(io::ErrorKind::Other, "unknown error")))
                }
            }
        }
    }
    silly_format_iter(
        &mut EditorIter { editor: rl },
        io::stdout(),
        format_on_newline,
        if print_debug {
            Some(std::io::stderr())
        } else {
            None
        },
        parse,
    )
}
