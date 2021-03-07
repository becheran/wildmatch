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
//! assert!(WildMatch::new("cat").is_match("cat"));
//! assert!(WildMatch::new("*cat*").is_match("dog_cat_dog"));
//! assert!(WildMatch::new("c?t").is_match("cat"));
//! assert!(WildMatch::new("c?t").is_match("cot"));
//! ```
//! Examples not matching wildcards:
//! ``` rust
//! # extern crate wildmatch; use wildmatch::WildMatch;
//! assert!(!WildMatch::new("dog").is_match("cat"));
//! assert!(!WildMatch::new("*d").is_match("cat"));
//! assert!(!WildMatch::new("????").is_match("cat"));
//! assert!(!WildMatch::new("?").is_match("cat"));
//! ```

use std::fmt;

/// Wildcard matcher used to match strings.
#[derive(Debug, Clone, Default)]
pub struct WildMatch {
    pattern: Vec<State>,
}

#[derive(Debug, Clone)]
struct State {
    next_char: Option<char>,
    has_wildcard: bool,
}

impl fmt::Display for WildMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        for state in &self.pattern {
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
        let mut simplified: Vec<State> = Vec::with_capacity(pattern.len());
        let mut prev_was_star = false;
        for current_char in pattern.chars() {
            match current_char {
                '*' => {
                    prev_was_star = true;
                }
                _ => {
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
        }
    }

    /// Indicates whether the matcher finds a match in the input string.
    pub fn is_match(&self, input: &str) -> bool {
        if self.pattern.is_empty() {
            return input.is_empty() 
        }
        let mut pattern_idx = 0;
        for input_char in input.chars() {
            match self.pattern.get(pattern_idx) {
                None => {
                    return false;
                }
                Some(p) if p.next_char == Some('?') || p.next_char == Some(input_char) => {
                    pattern_idx += 1;
                }
                Some(p) if p.has_wildcard => {
                    if p.next_char == None {
                        return true;
                    }
                }
                _ => {
                    // Go back to last state with wildcard
                    while let Some(pattern) = self.pattern.get(pattern_idx) {
                        if pattern.has_wildcard {
                            // Match last char again
                            if pattern.next_char == Some('?')
                                || pattern.next_char == Some(input_char)
                            {
                                pattern_idx += 1;
                            }
                            break;
                        }
                        if pattern_idx == 0 {
                            return false;
                        };
                        pattern_idx -= 1;
                    }
                }
            }
        }
        self.pattern[pattern_idx].next_char.is_none()
    }
}

impl<'a> PartialEq<&'a str> for WildMatch {
    fn eq(&self, &other: &&'a str) -> bool {
        self.is_match(other)
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
        assert!(m.is_match("cat"));
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
        assert_false!(m.is_match("cat"));
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
        assert_false!(m.is_match(expected))
    }

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
        assert!(m.is_match(expected))
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
    fn print_string() {
        let m = WildMatch::new("Foo/Bar");
        assert_eq!("Foo/Bar", m.to_string());
    }

    #[test]
    fn to_string_f() {
        let m = WildMatch::new("F");
        assert_eq!("F", m.to_string());
    }

    #[test]
    fn to_string_empty() {
        let m = WildMatch::new("");
        assert_eq!("", m.to_string());
    }
}
