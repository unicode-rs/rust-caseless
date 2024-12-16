use std::char;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Case folding a single code point can give up to this many code points.
const MAX_FOLDED_CODE_POINTS: usize = 3;

fn main() {
    let mut lines = include_str!("../CaseFolding.txt").lines();
    let first_line = lines.next().unwrap();
    let (major, minor, patch) = parse_version(first_line);

    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("case_folding_data.rs");
    let f = &mut File::create(&dst).unwrap();

    // Shorthand for `write!(f, ...).unwrap()`
    macro_rules! w {
        ($($args: tt)+) => { (write!(f, $($args)+)).unwrap(); }
    };

    w!("pub const UNICODE_VERSION: (u64, u64, u64) = ({}, {}, {});\n", major, minor, patch);
    w!("const CASE_FOLDING_TABLE: &'static [(char, [char; 3])] = &[\n");

    for line in lines {
        // Parse line. Skip if line is empty (or only comment). Skip if status is not F or C
        if let Some((from, _, to)) = parse_line(line).filter(status_is_f_or_c) {
            assert!(to.len() <= MAX_FOLDED_CODE_POINTS);
            let blanks = MAX_FOLDED_CODE_POINTS - to.len();

            // Write line
            let mut to = to.into_iter();
            let first_to = to.next().unwrap();
            w!("  ('{}', ['{}'", from, first_to);
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

fn parse_version(first_line: &str) -> (u64, u64, u64) {
    let (prefix, rest) = first_line.split_at(14);
    assert_eq!(prefix, "# CaseFolding-");

    let (rest, suffix) = rest.split_at(rest.len() - 4);
    assert_eq!(suffix, ".txt");

    let unicode_version: Vec<&str> = rest.split('.').collect();
    assert_eq!(unicode_version.len(), 3);
    assert!(unicode_version
        .iter()
        .all(|part| part.chars().all(|c| c.is_ascii_digit())));

    let (major, minor, patch): (u64, u64, u64) = (
        unicode_version[0].parse().unwrap(),
        unicode_version[1].parse().unwrap(),
        unicode_version[2].parse().unwrap(),
    );

    (major, minor, patch)
}

fn parse_line(line: &str) -> Option<(String, char, Vec<String>)> {
    // Handle comments: find content before the first # char (or whole line if there is no 3 char)
    let pre_comment = if line.contains('#') {
        line.split_once('#').unwrap().0
    } else {
        line
    };

    // Skip line if non-comment content is empty
    if pre_comment.is_empty() {
        return None;
    }

    let parts: Vec<&str> = pre_comment.split("; ").collect();
    assert!(parts.len() == 4);
    assert!(["C", "F", "S", "T"].contains(&parts[1]));
    assert!(parts[3] == "");

    let from = hex_to_escaped(parts[0]);
    let status = parts[1].chars().next().unwrap();
    let to = parts[2].split(' ').map(hex_to_escaped).collect::<Vec<_>>();

    return Some((from, status, to));
}

fn status_is_f_or_c((_to, status, _from): &(String, char, Vec<String>)) -> bool {
    *status == 'F' || *status == 'C'
}
