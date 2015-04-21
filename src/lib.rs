use unicode_normalization::UnicodeNormalization;

extern crate unicode_normalization;

include!(concat!(env!("OUT_DIR"), "/case_folding_data.rs"));

pub fn default_case_fold_char(c: char) -> CaseFoldingResult {
    match CASE_FOLDING_TABLE.binary_search_by(|&(x, _)| x.cmp(&c)) {
        Err(_) => CaseFoldingResult::Unchanged,
        Ok(i) => CaseFoldingResult::ReplacedWith(CASE_FOLDING_TABLE[i].1),
    }
}

#[derive(Copy, Clone)]
pub enum CaseFoldingResult {
    /// A `char` case folds to itself
    Unchanged,
    /// A `char` case folds to a sequence of one (most common),
    /// two, or three `char`s.
    ReplacedWith(&'static [char]),
}

pub struct CaseFold<I> {
    chars: I,
    queue: &'static [char],
}

impl<I> Iterator for CaseFold<I> where I: Iterator<Item = char> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if let Some(&c) = self.queue.first() {
            self.queue = &self.queue[1..];
            return Some(c);
        }
        self.chars.next().map(|c| match default_case_fold_char(c) {
            CaseFoldingResult::Unchanged => c,
            CaseFoldingResult::ReplacedWith(replacement) => {
                self.queue = &replacement[1..];
                replacement[0]
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.chars.size_hint();
        (low, high.and_then(|h| h.checked_mul(3)))
    }
}

pub trait Caseless {
    fn default_case_fold(self) -> CaseFold<Self>;
    fn default_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool;
    fn canonical_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool;
    fn compatibility_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool;
}

impl<I: Iterator<Item=char>> Caseless for I {
    fn default_case_fold(self) -> CaseFold<I> {
        CaseFold {
            chars: self,
            queue: &[],
        }
    }

    fn default_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool {
        iter_eq(self.default_case_fold(),
                other.default_case_fold())
    }

    fn canonical_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool {
        // FIXME: Inner NFD can be optimized:
        // "Normalization is not required before case folding,
        //  except for the character U+0345 "combining greek ypogegrammeni"
        //  and any characters that have it as part of their canonical decomposition,
        //  such as U+1FC3 "greek small letter eta with ypogegrammeni".
        //  In practice, optimized versions of canonical caseless matching
        //  can catch these special cases, thereby avoiding an extra normalization
        //  step for each comparison."
        // Unicode Standard, section 3.13 Default Case Algorithms
        iter_eq(self.nfd().default_case_fold().nfd(),
                other.nfd().default_case_fold().nfd())
    }

    fn compatibility_caseless_match<J: Iterator<Item=char>>(self, other: J) -> bool {
        // FIXME: Unclear if the inner NFD can be optimized here like in canonical_caseless_match.
        iter_eq(self.nfd().default_case_fold().nfkd().default_case_fold().nfkd(),
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

fn iter_eq<L: Iterator, R: Iterator>(mut a: L, mut b: R) -> bool where L::Item: PartialEq<R::Item> {
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (None, _) | (_, None) => return false,
            (Some(x), Some(y)) => if !x.eq(&y) { return false },
        }
    }
}
