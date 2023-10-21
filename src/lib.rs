//! Match strings against a simple wildcard pattern. Tests a wildcard
//! pattern `p` against an input string `s`. Returns true only when `p`
//! matches the entirety of `s`.
//!
//! See also the example described on
//! [wikipedia](https://en.wikipedia.org/wiki/Matching_wildcards) for
//! matching wildcards.
//!
//! No escape characters are defined.
//!
//! - `?` matches exactly one occurrence of any character.
//! - `*` matches arbitrary many (including zero) occurrences of any
//!   character.
//!
//! Examples matching wildcards:
//! ``` rust
//! # extern crate wildmatch; use wildmatch::GlobWildMatch;
//! assert!(GlobWildMatch::new("cat").matches("cat"));
//! assert!(GlobWildMatch::new("*cat*").matches("dog_cat_dog"));
//! assert!(GlobWildMatch::new("c?t").matches("cat"));
//! assert!(GlobWildMatch::new("c?t").matches("cot"));
//! ```
//! Examples not matching wildcards:
//! ``` rust
//! # extern crate wildmatch; use wildmatch::GlobWildMatch;
//! assert!(!GlobWildMatch::new("dog").matches("cat"));
//! assert!(!GlobWildMatch::new("*d").matches("cat"));
//! assert!(!GlobWildMatch::new("????").matches("cat"));
//! assert!(!GlobWildMatch::new("?").matches("cat"));
//! ```
//! 
//! You can specify custom `char` values for the single and multi-character
//! wildcards. For example, to use `%` as the multi-character wildcard and
//! `_` as the single-character wildcard:
//! ```rust
//! # extern crate wildmatch; use wildmatch::WildMatch;
//! assert!(WildMatch::<'%', '_'>::new("%cat%").matches("dog_cat_dog"));
//! ```

use std::fmt;

/// A wildcard matcher using `*` as the multi-character wildcard and `?` as
/// the single-character wildcard.
pub type GlobWildMatch = WildMatch<'*', '?'>;

/// Wildcard matcher used to match strings.
/// 
/// `MULTI_WILDCARD` is the character used to represent a
/// multiple-character wildcard (e.g., `*`), and `SINGLE_WILDCARD` is the
/// character used to represent a single-character wildcard (e.g., `?`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WildMatch<const MULTI_WILDCARD: char, const SINGLE_WILDCARD: char> {
    pattern: Vec<State>,
    max_questionmarks: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    next_char: Option<char>,
    has_wildcard: bool,
}

impl <const MULTI_WILDCARD: char, const SINGLE_WILDCARD: char> fmt::Display for WildMatch<MULTI_WILDCARD, SINGLE_WILDCARD> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        for state in &self.pattern {
            if state.has_wildcard {
                f.write_char(MULTI_WILDCARD)?;
            }
            if let Some(c) = state.next_char {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

impl <const MULTI_WILDCARD: char, const SINGLE_WILDCARD: char> WildMatch<MULTI_WILDCARD, SINGLE_WILDCARD> {
    /// Constructor with pattern which can be used for matching.
    pub fn new(pattern: &str) -> WildMatch<MULTI_WILDCARD, SINGLE_WILDCARD> {
        let mut simplified: Vec<State> = Vec::with_capacity(pattern.len());
        let mut prev_was_star = false;
        let mut max_questionmarks: usize = 0;
        let mut questionmarks: usize = 0;
        for current_char in pattern.chars() {
            if current_char == MULTI_WILDCARD {
                prev_was_star = true;
                max_questionmarks = std::cmp::max(max_questionmarks, questionmarks);
                questionmarks = 0;
            } else {
                if current_char == SINGLE_WILDCARD {
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
        let mut questionmark_matches: Vec<char> = Vec::with_capacity(self.max_questionmarks);
        for input_char in input.chars() {
            match self.pattern.get(pattern_idx) {
                None => {
                    return false;
                }
                Some(p) if p.next_char == Some(SINGLE_WILDCARD) => {
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
                            if self.pattern[pattern_idx].next_char == Some(SINGLE_WILDCARD) {
                                pattern_idx += 1;
                                continue;
                            }
                            let mut prev_input_char = prev_state.next_char;
                            if prev_input_char == Some(SINGLE_WILDCARD) {
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
                    if self.pattern[pattern_idx].next_char == Some(SINGLE_WILDCARD)
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

impl<'a, const MULTI_WILDCARD: char, const SINGLE_WILDCARD: char> PartialEq<&'a str> for WildMatch<MULTI_WILDCARD, SINGLE_WILDCARD> {
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
        let m = GlobWildMatch::new(pattern);
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
        let m = GlobWildMatch::new(pattern);
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
        let m = GlobWildMatch::new(pattern);
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
        let m = GlobWildMatch::new(pattern);
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
        let m = GlobWildMatch::new(COMPLEX_PATTERN);
        assert!(m.matches(TEXT));
    }

    #[test]
    fn complex_pattern_alternative_wildcards() {
        const TEXT: &str = "Lorem ipsum dolor sit amet, \
        consetetur sadipscing elitr, sed diam nonumy eirmod tempor \
        invidunt ut labore et dolore magna aliquyam erat, sed diam \
        voluptua. At vero eos et accusam et justo duo dolores et ea \
        rebum. Stet clita kasd gubergren, no sea takimata sanctus est \
        Lorem ipsum dolor sit amet.";
        const COMPLEX_PATTERN: &str = "Lorem_ipsum%dolore%ea% _____ata%.";
        let m = WildMatch::<'%', '_'>::new(COMPLEX_PATTERN);
        assert!(m.matches(TEXT));
    }

    #[test]
    fn compare_via_equal() {
        let m = GlobWildMatch::new("c?*");
        assert!(m == "cat");
        assert!(m == "car");
        assert!(m != "dog");
    }

    #[test]
    fn compare_empty() {
        let m: GlobWildMatch = GlobWildMatch::new("");
        assert!(m != "bar");
        assert!(m == "");
    }

    #[test]
    fn compare_default() {
        let m: GlobWildMatch = Default::default();
        assert!(m == "");
        assert!(m != "bar");
    }

    #[test]
    fn compare_wild_match() {
        assert_eq!(GlobWildMatch::default(), GlobWildMatch::new(""));
        assert_eq!(GlobWildMatch::new("abc"), GlobWildMatch::new("abc"));
        assert_eq!(GlobWildMatch::new("a*bc"), GlobWildMatch::new("a*bc"));
        assert_ne!(GlobWildMatch::new("abc"), GlobWildMatch::new("a*bc"));
        assert_ne!(GlobWildMatch::new("a*bc"), GlobWildMatch::new("a?bc"));
        assert_eq!(GlobWildMatch::new("a***c"), GlobWildMatch::new("a*c"));
    }

    #[test]
    fn print_string() {
        let m = GlobWildMatch::new("Foo/Bar");
        assert_eq!("Foo/Bar", m.to_string());
    }

    #[test]
    fn to_string_f() {
        let m = GlobWildMatch::new("F");
        assert_eq!("F", m.to_string());
    }

    #[test]
    fn to_string_with_star() {
        assert_eq!("a*bc", GlobWildMatch::new("a*bc").to_string());
        assert_eq!("a*bc", GlobWildMatch::new("a**bc").to_string());
        assert_eq!("a*bc*", GlobWildMatch::new("a*bc*").to_string());
    }

    #[test]
    fn to_string_with_question_sign() {
        assert_eq!("a?bc", GlobWildMatch::new("a?bc").to_string());
        assert_eq!("a??bc", GlobWildMatch::new("a??bc").to_string());
    }

    #[test]
    fn to_string_empty() {
        let m = GlobWildMatch::new("");
        assert_eq!("", m.to_string());
    }
}
