use atty::Stream;
use std::env;
use std::io;

use sillyfmt::silly_format;
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
    silly_format(
        io::stdin().lock(),
        io::stdout().lock(),
        format_on_newline,
        if print_debug {
            Some(std::io::stderr())
        } else {
            None
        },
        parse,
    )
}
