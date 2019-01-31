use std::io;

use sillyfmt::silly_format;

fn main() -> io::Result<()> {
    silly_format(io::stdin(), io::stdout())
}
