extern crate core;

use core::slice::BinarySearchResult;
use std::num::Int;
use std::char;

include!(concat!(env!("OUT_DIR"), "/case_folding_data.rs"));

pub fn default_case_fold_char(c: char) -> CaseFoldingResult {
    match CASE_FOLDING_TABLE.binary_search(|&(x, _)| x.cmp(&c)) {
        BinarySearchResult::NotFound(_) => CaseFoldingResult::Unchanged,
        BinarySearchResult::Found(i) => CaseFoldingResult::ReplacedWith(
            CASE_FOLDING_TABLE[i].1),
    }
}

#[deriving(Copy)]
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

pub fn default_case_fold<I>(chars: I) -> CaseFold<I> where I: Iterator<char> {
    CaseFold {
        chars: chars,
        queue: &[],
    }
}

impl<I> Iterator<char> for CaseFold<I> where I: Iterator<char> {
    fn next(&mut self) -> Option<char> {
        if let Some(&c) = self.queue.head() {
            self.queue = self.queue.tail();
            return Some(c);
        }
        self.chars.next().map(|c| match default_case_fold_char(c) {
            CaseFoldingResult::Unchanged => c,
            CaseFoldingResult::ReplacedWith(replacement) => {
                self.queue = replacement.tail();
                replacement[0]
            }
        })
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (low, high) = self.chars.size_hint();
        (low, high.and_then(|h| h.checked_mul(3)))
    }
}


pub fn default_case_fold_str(s: &str) -> String {
    default_case_fold(s.chars()).collect()
}


pub fn default_caseless_match<I, J>(i: I, j: J) -> bool
where I: Iterator<char>, J: Iterator<char> {
    iter_eq(default_case_fold(i),
            default_case_fold(j))
}

pub fn default_caseless_match_str(a: &str, b: &str) -> bool {
    default_caseless_match(a.chars(), b.chars())
}

pub fn canonical_caseless_match<I, J>(i: I, j: J) -> bool
where I: Iterator<char>, J: Iterator<char> {
    // FIXME: Inner NFD can be optimized:
    // "Normalization is not required before case folding,
    //  except for the character U+0345 "combining greek ypogegrammeni"
    //  and any characters that have it as part of their canonical decomposition,
    //  such as U+1FC3 "greek small letter eta with ypogegrammeni".
    //  In practice, optimized versions of canonical caseless matching
    //  can catch these special cases, thereby avoiding an extra normalization
    //  step for each comparison."
    // Unicode Standard, section 3.13 Default Case Algorithms
    iter_eq(nfd(default_case_fold(nfd(i))),
            nfd(default_case_fold(nfd(j))))
}

pub fn canonical_caseless_match_str(a: &str, b: &str) -> bool {
    canonical_caseless_match(a.chars(), b.chars())
}

pub fn compatibility_caseless_match<I, J>(i: I, j: J) -> bool
where I: Iterator<char>, J: Iterator<char> {
    // FIXME: Unclear if the inner NFD can be optimized here like in canonical_caseless_match.
    iter_eq(nfkd(default_case_fold(nfkd(default_case_fold(nfd(i))))),
            nfkd(default_case_fold(nfkd(default_case_fold(nfd(j))))))
}

pub fn compatibility_caseless_match_str(a: &str, b: &str) -> bool {
    compatibility_caseless_match(a.chars(), b.chars())
}


/// Like `i.collect::Vec<_>() == j.collect::Vec<_>()`, but does not allocate.
pub fn iter_eq<I, J, E>(i: I, j: J) -> bool where I: Iterator<E>, J: Iterator<E>, E: Eq {
    zip_all(i, j, |a, b| a == b)
}


/// Like `i.zip(j).all(f)`,
/// but also return `false` in the iterators don’t have the same length.
/// FIXME: Add `zip_any`?
pub fn zip_all<I, J, A, B>(mut i: I, mut j: J, f: |A, B| -> bool) -> bool
where I: Iterator<A>, J: Iterator<B> {
    loop {
        match (i.next(), j.next()) {
            (None, None) => return true,
            (Some(a), Some(b)) => {
                if !f(a, b) {
                    return false
                }
            }
            _ => return false,
        }
    }
}


fn nfd<I>(chars: I) -> Decompositions<I> where I: Iterator<char> {
    Decompositions {
        iter: chars,
        buffer: Vec::new(),
        sorted: false,
        kind: DecompositionType::Canonical
    }
}

fn nfkd<I>(chars: I) -> Decompositions<I> where I: Iterator<char> {
    Decompositions {
        iter: chars,
        buffer: Vec::new(),
        sorted: false,
        kind: DecompositionType::Compatible
    }
}


// The rest of the file is taken from libcollections,
// but with Decompositions::iter changed to be any Iterator<char>
// instead of Chars<'a>
// FIXME: expose a generic API, in the spirit of PR #19042 ?
// Caseless matching demonstrates a use case for
// normalizing iterators rather than &str,
// there could be others.

struct Decompositions<I> {
    kind: DecompositionType,
    iter: I,
    buffer: Vec<(char, u8)>,
    sorted: bool
}

impl<I> Iterator<char> for Decompositions<I> where I: Iterator<char> {
    #[inline]
    fn next(&mut self) -> Option<char> {
        match self.buffer.as_slice().head() {
            Some(&(c, 0)) => {
                self.sorted = false;
                self.buffer.remove(0);
                return Some(c);
            }
            Some(&(c, _)) if self.sorted => {
                self.buffer.remove(0);
                return Some(c);
            }
            _ => self.sorted = false
        }

        let decomposer = match self.kind {
            DecompositionType::Canonical => char::decompose_canonical,
            DecompositionType::Compatible => char::decompose_compatible
        };

        if !self.sorted {
            for ch in self.iter {
                let buffer = &mut self.buffer;
                let sorted = &mut self.sorted;
                decomposer(ch, |d| {
                    let class = char::canonical_combining_class(d);
                    if class == 0 && !*sorted {
                        canonical_sort(buffer.as_mut_slice());
                        *sorted = true;
                    }
                    buffer.push((d, class));
                });
                if *sorted { break }
            }
        }

        if !self.sorted {
            canonical_sort(self.buffer.as_mut_slice());
            self.sorted = true;
        }

        match self.buffer.remove(0) {
            Some((c, 0)) => {
                self.sorted = false;
                Some(c)
            }
            Some((c, _)) => Some(c),
            None => None
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (lower, _) = self.iter.size_hint();
        (lower, None)
    }
}

enum DecompositionType {
    Canonical,
    Compatible
}

// Helper functions used for Unicode normalization
fn canonical_sort(comb: &mut [(char, u8)]) {
    let len = comb.len();
    for i in range(0, len) {
        let mut swapped = false;
        for j in range(1, len-i) {
            let class_a = comb[j-1].1;
            let class_b = comb[j].1;
            if class_a != 0 && class_b != 0 && class_a > class_b {
                comb.swap(j-1, j);
                swapped = true;
            }
        }
        if !swapped { break; }
    }
}
