#[macro_use]
extern crate afl;

use std::collections::HashMap;

use sillyfmt::do_format;
use sillyfmt_tree_sitter::parse;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = String::from_utf8(data.to_vec()) {
            let mut output_buffer = Vec::with_capacity(1024 * 1024);
            do_format(&mut output_buffer, s.clone(), parse).unwrap();

            let required: HashMap<char, usize> =
                s.chars()
                    .filter(|c| c.is_alphanumeric())
                    .fold(HashMap::new(), |mut m, c| {
                        *m.entry(c).or_insert(0) += 1;
                        m
                    });
            let observed: HashMap<char, usize> = String::from_utf8_lossy(&output_buffer)
                .chars()
                .fold(HashMap::new(), |mut m, c| {
                    *m.entry(c).or_insert(0) += 1;
                    m
                });

            for (c, count) in required {
                let observed_count = observed.get(&c).cloned().unwrap_or(0);
                assert!(
                    observed_count >= count,
                    "Missing {} occurences of '{}' (original: {}, output: {})",
                    count - observed_count,
                    c,
                    s,
                    String::from_utf8_lossy(&output_buffer)
                );
            }
        }
    });
}
