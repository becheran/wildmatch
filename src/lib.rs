//! Match strings against a simple wildcard pattern.
//! Tests a wildcard pattern p against an input string s. Returns true only when p matches the entirety of s.
//!
//! A very simplified syntax is used here. See also the example described on [wikipedia](https://en.wikipedia.org/wiki/Matching_wildcards) for matching wildcards.
//!
//! No escape characters are defined
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

/// Wildcard matcher used to match strings.
#[derive(Debug)]
pub struct WildMatch {
    pattern: Vec<State>,
}

#[derive(Debug)]
struct State {
    next_char: char,
    has_wildcard: bool,
    is_final_state: bool,
}

impl WildMatch {
    /// Constructor with pattern which can be used for matching.
    pub fn new(pattern: &str) -> WildMatch {
        let mut simplified: Vec<State> = Vec::new();
        let mut prev_was_star = false;
        for current_char in pattern.chars() {
            match current_char {
                '*' => {
                    prev_was_star = true;
                }
                _ => {
                    let s = State {
                        next_char: current_char,
                        has_wildcard: prev_was_star,
                        is_final_state: false,
                    };
                    simplified.push(s);
                    prev_was_star = false;
                }
            }
        }

        if pattern.chars().count() > 0 {
            let final_state = State {
                next_char: '*', // Sign does not matter
                has_wildcard: prev_was_star,
                is_final_state: true,
            };
            simplified.push(final_state);
        }

        WildMatch {
            pattern: simplified,
        }
    }

    /// Indicates whether the matcher finds a match in the input string.
    pub fn is_match(&self, input: &str) -> bool {
        let mut pattern_idx = 0;
        for input_char in input.chars() {
            match self.pattern.get(pattern_idx) {
                None => {
                    return false;
                }
                Some(c) if c.next_char == '?' => {
                    if !c.is_final_state{
                        pattern_idx += 1;
                    }
                }
                Some(c) if c.next_char == input_char => {
                    if !c.is_final_state{
                        pattern_idx += 1;
                    }
                }
                Some(c) if c.has_wildcard => {
                    continue;
                }
                _ => {
                    // Go back to last state with wildcard
                    while let Some(pattern) = self.pattern.get(pattern_idx) {
                        if pattern.has_wildcard {
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
        return self.pattern.get(pattern_idx).unwrap().is_final_state;
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
    #[test_case("???", "wildcats")]
    fn no_match_long(pattern: &str, expected: &str) {
        let m = WildMatch::new(pattern);
        assert_false!(m.is_match(expected))
    }

    #[test_case("*cat*", "d&(*og_cat_dog")]
    #[test_case("*?*", "d&(*og_cat_dog")]
    #[test_case("*a*", "d&(*og_cat_dog")]
    #[test_case("*o?", "hog_cat_dog")]
    #[test_case("*o?", "cat_dog")]
    #[test_case("*at_dog", "cat_dog")]
    #[test_case(" ", " ")]
    #[test_case("* ", "\n ")]
    #[test_case("\n", "\n", name="special_chars")]
    fn match_long(pattern: &str, expected: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.is_match(expected))
    }
}
