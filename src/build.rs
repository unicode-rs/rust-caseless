extern crate regex;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::char;
use regex::Regex;

fn main() {
    let mut lines = include_str!("../CaseFolding.txt").lines();
    let first_line = lines.next().unwrap();
    let version_regex = Regex::new(r"^# CaseFolding-(\d.\d.\d).txt$").unwrap();
    let unicode_version = version_regex.captures(first_line).unwrap().at(1).unwrap();

    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("case_folding_data.rs");
    let mut f = &mut File::create(&dst).unwrap();

    macro_rules! w {
        ($($args: tt)+) => { (write!(f, $($args)+)).unwrap(); }
    };

    w!("pub const UNICODE_VERSION: &'static str = \"{}\";\n", unicode_version);
    w!("const CASE_FOLDING_TABLE: &'static [(char, &'static [char])] = &[\n");

    // Entry with C (common case folding) or F (full case folding) status
    let c_or_f_entry = Regex::new(r"^([0-9A-F]+); [CF]; ([0-9A-F ]+);").unwrap();

    for line in lines {
        if let Some(captures) = c_or_f_entry.captures(line) {
            let from = captures.at(1).unwrap();
            let mut to = captures.at(2).unwrap().split(' ');
            let first_to = to.next().unwrap();
            w!("  ('{}', &['{}'", hex_to_escaped(from), hex_to_escaped(first_to));
            for c in to {
                w!(", '{}'", hex_to_escaped(c));
            }
            w!("]),\n");
        }
    }
    w!("];\n");
}


fn hex_to_escaped(hex: &str) -> String {
    char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap().escape_default().collect()
}
