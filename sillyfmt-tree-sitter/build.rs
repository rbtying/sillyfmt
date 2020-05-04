extern crate cc;

use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["..", "tree-sitter-sillyfmt", "src"].iter().collect();
    println!(
        "cargo:rerun-if-changed={}",
        dir.join("parser.c").to_string_lossy()
    );
    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-sillyfmt")
}
