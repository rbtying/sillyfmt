use std::collections::HashMap;
use std::io::{self, Read};

use sillyfmt::do_format;

fn main() -> io::Result<()> {
    let mut output_buffer = Vec::with_capacity(1024 * 1024);
    let mut s = String::new();
    if io::stdin().read_to_string(&mut s).is_ok() {
        do_format(&mut output_buffer, s.clone()).unwrap();

        let required: HashMap<char, usize> =
            s.chars()
                .filter(|c| !c.is_whitespace())
                .fold(HashMap::new(), |mut m, c| {
                    *m.entry(c).or_insert(0) += 1;
                    m
                });
        let observed: HashMap<char, usize> =
            String::from_utf8_lossy(&output_buffer)
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

    Ok(())
}
