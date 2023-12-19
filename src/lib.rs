//! Match strings against a simple wildcard pattern.
//! Tests a wildcard pattern `p` against an input string `s`. Returns true only when `p` matches the entirety of `s`.
//!
//! See also the example described on [wikipedia](https://en.wikipedia.org/wiki/Matching_wildcards) for matching wildcards.
//!
//! No escape characters are defined.
//!
//! - `?` matches exactly one occurrence of any character.
//! - `*` matches arbitrary many (including zero) occurrences of any character.
//!
//! Examples matching wildcards:
//! ``` rust
//! # extern crate wildmatch; use wildmatch::WildMatch;
//! assert!(WildMatch::new("cat").matches("cat"));
//! assert!(WildMatch::new("*cat*").matches("dog_cat_dog"));
//! assert!(WildMatch::new("c?t").matches("cat"));
//! assert!(WildMatch::new("c?t").matches("cot"));
//! ```
//! Examples not matching wildcards:
//! ``` rust
//! # extern crate wildmatch; use wildmatch::WildMatch;
//! assert!(!WildMatch::new("dog").matches("cat"));
//! assert!(!WildMatch::new("*d").matches("cat"));
//! assert!(!WildMatch::new("????").matches("cat"));
//! assert!(!WildMatch::new("?").matches("cat"));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "no_std"))]
compile_error!("Exactly one of the feature \"std\" or \"no_std\" must be enabled; both are.");
#[cfg(not(any(feature = "std", feature = "no_std")))]
compile_error!("Exactly one of the feature \"std\" or \"no_std\" must be enabled; none is.");

#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(not(feature = "std"))]
use tinyvec::TinyVec as Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Wildcard matcher used to match strings.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WildMatch {
    #[cfg(feature = "std")]
    pattern: Vec<State>,
    #[cfg(not(feature = "std"))]
    pattern: Vec<[State; 32]>,
    max_questionmarks: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
struct State {
    next_char: Option<char>,
    has_wildcard: bool,
}

impl fmt::Display for WildMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write;

        for state in &self.pattern {
            if state.has_wildcard {
                f.write_char('*')?;
            }
            if let Some(c) = state.next_char {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

impl WildMatch {
    /// Constructor with pattern which can be used for matching.
    pub fn new(pattern: &str) -> WildMatch {
        let mut simplified = Vec::with_capacity(pattern.len());
        let mut prev_was_star = false;
        let mut max_questionmarks: usize = 0;
        let mut questionmarks: usize = 0;
        for current_char in pattern.chars() {
            match current_char {
                '*' => {
                    prev_was_star = true;
                    max_questionmarks = core::cmp::max(max_questionmarks, questionmarks);
                    questionmarks = 0;
                }
                _ => {
                    if current_char == '?' {
                        questionmarks += 1;
                    }
                    let s = State {
                        next_char: Some(current_char),
                        has_wildcard: prev_was_star,
                    };
                    simplified.push(s);
                    prev_was_star = false;
                }
            }
        }

        if !pattern.is_empty() {
            let final_state = State {
                next_char: None,
                has_wildcard: prev_was_star,
            };
            simplified.push(final_state);
        }

        WildMatch {
            pattern: simplified,
            max_questionmarks,
        }
    }

    #[deprecated(since = "2.0.0", note = "use `matches` instead")]
    pub fn is_match(&self, input: &str) -> bool {
        self.matches(input)
    }

    /// Returns true if pattern applies to the given input string
    pub fn matches(&self, input: &str) -> bool {
        if self.pattern.is_empty() {
            return input.is_empty();
        }
        let mut pattern_idx = 0;
        const NONE: usize = usize::MAX;
        let mut last_wildcard_idx = NONE;
        #[cfg(feature = "std")]
        let mut questionmark_matches = Vec::<char>::with_capacity(self.max_questionmarks);
        #[cfg(not(feature = "std"))]
        let mut questionmark_matches = Vec::<[char; 10]>::with_capacity(self.max_questionmarks);
        for input_char in input.chars() {
            match self.pattern.get(pattern_idx) {
                None => {
                    return false;
                }
                Some(p) if p.next_char == Some('?') => {
                    if p.has_wildcard {
                        last_wildcard_idx = pattern_idx;
                    }
                    pattern_idx += 1;
                    questionmark_matches.push(input_char);
                }
                Some(p) if p.next_char == Some(input_char) => {
                    if p.has_wildcard {
                        last_wildcard_idx = pattern_idx;
                        questionmark_matches.clear();
                    }
                    pattern_idx += 1;
                }
                Some(p) if p.has_wildcard => {
                    if p.next_char == None {
                        return true;
                    }
                }
                _ => {
                    if last_wildcard_idx == NONE {
                        return false;
                    }
                    if !questionmark_matches.is_empty() {
                        // Try to match a different set for questionmark
                        let mut questionmark_idx = 0;
                        let current_idx = pattern_idx;
                        pattern_idx = last_wildcard_idx;
                        for prev_state in self.pattern[last_wildcard_idx + 1..current_idx].iter() {
                            if self.pattern[pattern_idx].next_char == Some('?') {
                                pattern_idx += 1;
                                continue;
                            }
                            let mut prev_input_char = prev_state.next_char;
                            if prev_input_char == Some('?') {
                                prev_input_char = Some(questionmark_matches[questionmark_idx]);
                                questionmark_idx += 1;
                            }
                            if self.pattern[pattern_idx].next_char == prev_input_char {
                                pattern_idx += 1;
                            } else {
                                pattern_idx = last_wildcard_idx;
                                questionmark_matches.clear();
                                break;
                            }
                        }
                    } else {
                        // Directly go back to the last wildcard
                        pattern_idx = last_wildcard_idx;
                    }

                    // Match last char again
                    if self.pattern[pattern_idx].next_char == Some('?')
                        || self.pattern[pattern_idx].next_char == Some(input_char)
                    {
                        pattern_idx += 1;
                    }
                }
            }
        }
        self.pattern[pattern_idx].next_char.is_none()
    }
}

impl<'a> PartialEq<&'a str> for WildMatch {
    fn eq(&self, &other: &&'a str) -> bool {
        self.matches(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::assert_false;
    use ntest::test_case;

    #[test_case("**")]
    #[test_case("*")]
    #[test_case("*?*")]
    #[test_case("c*")]
    #[test_case("c?*")]
    #[test_case("???")]
    #[test_case("c?t")]
    #[test_case("cat")]
    #[test_case("*cat")]
    #[test_case("cat*")]
    fn is_match(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.matches("cat"));
    }

    #[test_case("*d*")]
    #[test_case("*d")]
    #[test_case("d*")]
    #[test_case("*c")]
    #[test_case("?")]
    #[test_case("??")]
    #[test_case("????")]
    #[test_case("?????")]
    #[test_case("*????")]
    #[test_case("cats")]
    #[test_case("cat?")]
    #[test_case("cacat")]
    #[test_case("cat*dog")]
    fn no_match(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert_false!(m.matches("cat"));
    }

    #[test_case("cat?", "wildcats")]
    #[test_case("cat*", "wildcats")]
    #[test_case("*x*", "wildcats")]
    #[test_case("*a", "wildcats")]
    #[test_case("", "wildcats")]
    #[test_case(" ", "wildcats")]
    #[test_case(" ", "\n")]
    #[test_case(" ", "\t", name = "whitespaceMismatch")]
    #[test_case("???", "wildcats")]
    fn no_match_long(pattern: &str, expected: &str) {
        let m = WildMatch::new(pattern);
        assert_false!(m.matches(expected))
    }

    #[test_case("*???a", "bbbba")]
    #[test_case("*???a", "bbbbba")]
    #[test_case("*???a", "bbbbbba")]
    #[test_case("*o?a*", "foobar")]
    #[test_case("*ooo?ar", "foooobar")]
    #[test_case("*o?a*r", "foobar")]
    #[test_case("*cat*", "d&(*og_cat_dog")]
    #[test_case("*?*", "d&(*og_cat_dog")]
    #[test_case("*a*", "d&(*og_cat_dog")]
    #[test_case("*", "*")]
    #[test_case("*", "?")]
    #[test_case("?", "?")]
    #[test_case("wildcats", "wildcats")]
    #[test_case("wild*cats", "wild?cats")]
    #[test_case("wi*ca*s", "wildcats")]
    #[test_case("wi*ca?s", "wildcats")]
    #[test_case("*o?", "hog_cat_dog")]
    #[test_case("*o?", "cat_dog")]
    #[test_case("*at_dog", "cat_dog")]
    #[test_case(" ", " ")]
    #[test_case("* ", "\n ")]
    #[test_case("\n", "\n", name = "special_chars")]
    #[test_case("*32", "432")]
    #[test_case("*32", "332")]
    #[test_case("*332", "332")]
    #[test_case("*32", "32")]
    #[test_case("*32", "3232")]
    #[test_case("*32", "3232332")]
    #[test_case("*?2", "332")]
    #[test_case("*?2", "3332")]
    #[test_case("33*", "333")]
    #[test_case("da*da*da*", "daaadabadmanda")]
    #[test_case("*?", "xx")]
    fn match_long(pattern: &str, expected: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.matches(expected));
    }

    #[test]
    fn complex_pattern() {
        const TEXT: &str = "Lorem ipsum dolor sit amet, \
        consetetur sadipscing elitr, sed diam nonumy eirmod tempor \
        invidunt ut labore et dolore magna aliquyam erat, sed diam \
        voluptua. At vero eos et accusam et justo duo dolores et ea \
        rebum. Stet clita kasd gubergren, no sea takimata sanctus est \
        Lorem ipsum dolor sit amet.";
        const COMPLEX_PATTERN: &str = "Lorem?ipsum*dolore*ea* ?????ata*.";
        let m = WildMatch::new(COMPLEX_PATTERN);
        assert!(m.matches(TEXT));
    }

    #[test]
    fn compare_via_equal() {
        let m = WildMatch::new("c?*");
        assert!(m == "cat");
        assert!(m == "car");
        assert!(m != "dog");
    }

    #[test]
    fn compare_empty() {
        let m: WildMatch = WildMatch::new("");
        assert!(m != "bar");
        assert!(m == "");
    }

    #[test]
    fn compare_default() {
        let m: WildMatch = Default::default();
        assert!(m == "");
        assert!(m != "bar");
    }

    #[test]
    fn compare_wild_match() {
        assert_eq!(WildMatch::default(), WildMatch::new(""));
        assert_eq!(WildMatch::new("abc"), WildMatch::new("abc"));
        assert_eq!(WildMatch::new("a*bc"), WildMatch::new("a*bc"));
        assert_ne!(WildMatch::new("abc"), WildMatch::new("a*bc"));
        assert_ne!(WildMatch::new("a*bc"), WildMatch::new("a?bc"));
        assert_eq!(WildMatch::new("a***c"), WildMatch::new("a*c"));
        assert_eq!(WildMatch::new("a?bc"), WildMatch::new("a?bc"));
        assert_ne!(WildMatch::new("a??bc"), WildMatch::new("a?bc"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn print_string() {
        let m = WildMatch::new("Foo/Bar");
        assert_eq!("Foo/Bar", m.to_string());
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_string_f() {
        let m = WildMatch::new("F");
        assert_eq!("F", m.to_string());
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_string_with_star() {
        assert_eq!("a*bc", WildMatch::new("a*bc").to_string());
        assert_eq!("a*bc", WildMatch::new("a**bc").to_string());
        assert_eq!("a*bc*", WildMatch::new("a*bc*").to_string());
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_string_with_question_sign() {
        assert_eq!("a?bc", WildMatch::new("a?bc").to_string());
        assert_eq!("a??bc", WildMatch::new("a??bc").to_string());
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_string_empty() {
        let m = WildMatch::new("");
        assert_eq!("", m.to_string());
    }
}
