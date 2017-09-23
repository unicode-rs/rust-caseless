extern crate regex;

use std::char;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use regex::Regex;

// Case folding a single code point can give up to this many code points.
const MAX_FOLDED_CODE_POINTS: usize = 3;

fn main() {
    let mut lines = include_str!("../CaseFolding.txt").lines();
    let first_line = lines.next().unwrap();
    let version_regex = Regex::new(r"^# CaseFolding-(\d+)\.(\d+)\.(\d+).txt$").unwrap();
    let unicode_version = &version_regex.captures(first_line).unwrap();
    let (major, minor, patch): (u64, u64, u64) = (
        unicode_version[1].parse().unwrap(),
        unicode_version[2].parse().unwrap(),
        unicode_version[3].parse().unwrap(),
    );

    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("case_folding_data.rs");
    let f = &mut File::create(&dst).unwrap();

    macro_rules! w {
        ($($args: tt)+) => { (write!(f, $($args)+)).unwrap(); }
    };

    w!("pub const UNICODE_VERSION: (u64, u64, u64) = ({}, {}, {});\n", major, minor, patch);
    w!("const CASE_FOLDING_TABLE: &'static [(char, [char; 3])] = &[\n");

    // Entry with C (common case folding) or F (full case folding) status
    let c_or_f_entry = Regex::new(r"^([0-9A-F]+); [CF]; ([0-9A-F ]+);").unwrap();

    for line in lines {
        if let Some(captures) = c_or_f_entry.captures(line) {
            let from = &captures[1];
            let to = captures[2].split(' ').map(hex_to_escaped).collect::<Vec<_>>();
            assert!(to.len() <= MAX_FOLDED_CODE_POINTS);
            let blanks = MAX_FOLDED_CODE_POINTS - to.len();
            let mut to = to.into_iter();
            let first_to = to.next().unwrap();
            w!("  ('{}', ['{}'", hex_to_escaped(from), first_to);
            for c in to {
                w!(", '{}'", c);
            }
            for _ in 0..blanks {
                w!(", '\\0'");
            }
            w!("]),\n");
        }
    }
    w!("];\n");
}


fn hex_to_escaped(hex: &str) -> String {
    let c = u32::from_str_radix(hex, 16).unwrap();
    assert!(c != 0);
    char::from_u32(c).unwrap().escape_default().collect()
}
