use std::io::Cursor;

use sillyfmt::silly_format;
use wasm_bindgen::prelude::*;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn format(s: &str) -> String {
    let mut out = Vec::new();
    let _ = silly_format(Cursor::new(s), Cursor::new(&mut out), false);
    String::from_utf8_lossy(&out).to_string()
}
