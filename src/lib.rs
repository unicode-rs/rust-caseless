use unicode_normalization::UnicodeNormalization;
use std::cmp::Ordering;

extern crate unicode_normalization;

include!(concat!(env!("OUT_DIR"), "/case_folding_data.rs"));

pub trait Caseless {
    fn default_case_fold(self) -> CaseFold<Self> where Self: Sized;

    fn default_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool where Self: Sized {
        self.default_caseless_compare(other) == Ordering::Equal
    }

    fn canonical_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool where Self: Sized {
        self.canonical_caseless_compare(other) == Ordering::Equal
    }

    fn compatibility_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool where Self: Sized {
        self.compatibility_caseless_compare(other) == Ordering::Equal
    }

    fn default_caseless_compare<J: Iterator<Item=char>>(self, other: J) -> Ordering;

    fn canonical_caseless_compare<J: Iterator<Item=char>>(self, other: J) -> Ordering;

    fn compatibility_caseless_compare<J: Iterator<Item=char>>(self, other: J) -> Ordering;

    fn default_caseless_starts_with<J: Iterator<Item=char>>(self, other: J) -> bool;

    fn canonical_caseless_starts_with<J: Iterator<Item=char>>(self, other: J) -> bool;

    fn compatibility_caseless_starts_with<J: Iterator<Item=char>>(self, other: J) -> bool;
}

impl<I: Iterator<Item=char>> Caseless for I {
    fn default_case_fold(self) -> CaseFold<I> {
        CaseFold {
            chars: self,
            queue: ['\0', '\0'],
        }
    }

    fn default_caseless_compare<J: Iterator<Item=char>>(self, other: J) -> Ordering {
        iter_cmp(self.default_case_fold(),
                 other.default_case_fold())
    }

    fn canonical_caseless_compare<J: Iterator<Item=char>>(self, other: J) -> Ordering {
        // FIXME: Inner NFD can be optimized:
        // "Normalization is not required before case folding,
        //  except for the character U+0345 "combining greek ypogegrammeni"
        //  and any characters that have it as part of their canonical decomposition,
        //  such as U+1FC3 "greek small letter eta with ypogegrammeni".
        //  In practice, optimized versions of canonical caseless matching
        //  can catch these special cases, thereby avoiding an extra normalization
        //  step for each comparison."
        // Unicode Standard, section 3.13 Default Case Algorithms
        iter_cmp(self.nfd().default_case_fold().nfd(),
                 other.nfd().default_case_fold().nfd())
    }

    fn compatibility_caseless_compare<J: Iterator<Item=char>>(self, other: J) -> Ordering {
        // FIXME: Unclear if the inner NFD can be optimized here like in canonical_caseless_compare.
        iter_cmp(self.nfd().default_case_fold().nfkd().default_case_fold().nfkd(),
                 other.nfd().default_case_fold().nfkd().default_case_fold().nfkd())
    }

    fn default_caseless_starts_with<J: Iterator<Item=char>>(self, other: J) -> bool {
        iter_starts_with(
            self.default_case_fold(),
            other.default_case_fold())
    }

    fn canonical_caseless_starts_with<J: Iterator<Item=char>>(self, other: J) -> bool {
        // FIXME: Inner NFD can be optimized. See [canonical_caseless_compare]
        iter_starts_with(
            self.nfd().default_case_fold().nfd(),
            other.nfd().default_case_fold().nfd())
    }

    fn compatibility_caseless_starts_with<J: Iterator<Item=char>>(self, other: J) -> bool {
        // FIXME: Unclear if the inner NFD can be optimized here like in canonical_caseless_compare.
        iter_starts_with(
            self.nfd().default_case_fold().nfkd().default_case_fold().nfkd(),
            other.nfd().default_case_fold().nfkd().default_case_fold().nfkd())
    }
}

pub fn default_case_fold_str(s: &str) -> String {
    s.chars().default_case_fold().collect()
}

pub fn default_caseless_match_str(a: &str, b: &str) -> bool {
    a.chars().default_caseless_match(b.chars())
}

pub fn canonical_caseless_match_str(a: &str, b: &str) -> bool {
    a.chars().canonical_caseless_match(b.chars())
}

pub fn compatibility_caseless_match_str(a: &str, b: &str) -> bool {
    a.chars().compatibility_caseless_match(b.chars())
}

pub fn default_caseless_compare_str(a: &str, b: &str) -> Ordering {
    a.chars().default_caseless_compare(b.chars())
}

pub fn canonical_caseless_compare_str(a: &str, b: &str) -> Ordering {
    a.chars().canonical_caseless_compare(b.chars())
}

pub fn compatibility_caseless_compare_str(a: &str, b: &str) -> Ordering {
    a.chars().compatibility_caseless_compare(b.chars())
}

pub fn default_caseless_starts_with_str(a: &str, b: &str) -> bool {
    a.chars().default_caseless_starts_with(b.chars())
}

pub fn canonical_caseless_starts_with_str(a: &str, b: &str) -> bool {
    a.chars().canonical_caseless_starts_with(b.chars())
}

pub fn compatibility_caseless_starts_with_str(a: &str, b: &str) -> bool {
    a.chars().compatibility_caseless_starts_with(b.chars())
}

fn iter_cmp<L: Iterator, R: Iterator<Item = L::Item>>(mut a: L, mut b: R) -> Ordering where L::Item:  Ord {
    loop {
        match (a.next(), b.next()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) => if !x.eq(&y) { return x.cmp(&y) },
        }
    }
}

fn iter_starts_with<L: Iterator, R: Iterator<Item = L::Item>>(mut a: L, mut b: R) -> bool where L::Item: Ord + std::fmt::Debug {
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (None, _) => return false,
            (_, None) => return true,
            (Some(x), Some(y)) => if !x.eq(&y) { return false },
        }
    }
}

pub struct CaseFold<I> {
    chars: I,
    queue: [char; 2],
}

impl<I> Iterator for CaseFold<I> where I: Iterator<Item = char> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let c = self.queue[0];
        if c != '\0' {
            self.queue[0] = self.queue[1];
            self.queue[1] = '\0';
            return Some(c)
        }
        self.chars.next().map(|c| {
            match CASE_FOLDING_TABLE.binary_search_by(|&(x, _)| x.cmp(&c)) {
                Err(_) => c,
                Ok(i) => {
                    let folded = CASE_FOLDING_TABLE[i].1;
                    self.queue = [folded[1], folded[2]];
                    folded[0]
                }
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let queue_len = if self.queue[0] == '\0' {
            0
        } else if self.queue[1] == '\0' {
            1
        } else {
            2
        };
        let (low, high) = self.chars.size_hint();
        (low.saturating_add(queue_len),
         high.and_then(|h| h.checked_mul(3)).and_then(|h| h.checked_add(queue_len)))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use super::default_case_fold_str;
    use super::default_caseless_match_str;
    use super::default_caseless_compare_str;
    use super::default_caseless_starts_with_str;

    #[test]
    fn test_str_fold() {
        assert_eq!(default_case_fold_str("Test Case"), "test case");
        assert_eq!(default_case_fold_str("Teſt Caſe"), "test case");
        assert_eq!(default_case_fold_str("spiﬃest"), "spiffiest");
        assert_eq!(default_case_fold_str("straße"), "strasse");
    }

    #[test]
    fn test_str_match() {
        assert!(default_caseless_match_str("Test Case", "test case"));
        assert!(default_caseless_match_str("Teſt Caſe", "test case"));
        assert!(default_caseless_match_str("straße", "strasse"));
    }

    #[test]
    fn test_str_compare() {
        assert_eq!(default_caseless_compare_str("Test Case", "test"), Ordering::Greater);
        assert_eq!(default_caseless_compare_str("Teſt Caſe", "test"), Ordering::Greater);
        assert_eq!(default_caseless_compare_str("straße", "strass"), Ordering::Greater);
        assert_eq!(default_caseless_compare_str("Test Case", "case"), Ordering::Greater);
    }

    #[test]
    fn test_str_starts_with() {
        assert!(default_caseless_starts_with_str("Test Case", "test"));
        assert!(default_caseless_starts_with_str("Teſt Caſe", "test"));
        assert!(default_caseless_starts_with_str("straße", "strass"));
        assert!(!default_caseless_starts_with_str("Test Case", "case"));
    }
}

