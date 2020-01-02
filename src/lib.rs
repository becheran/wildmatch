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
//! assert!(WildMatch::new("cat").is_match("cat"));
//! assert!(WildMatch::new("*cat*").is_match("dog_cat_dog"));
//! assert!(WildMatch::new("c?t").is_match("cat"));
//! assert!(WildMatch::new("c?t").is_match("cot"));
//! ```
//! Examples not matching wildcards:
//! ``` rust
//! assert!(WildMatch::new("dog").is_match("cat"));
//! assert!(WildMatch::new("*d").is_match("cat"));
//! assert!(WildMatch::new("????").is_match("cat"));
//! assert!(WildMatch::new("?").is_match("cat"));
//! ```

/// Wildcard matcher used to match strings.
#[derive(Debug)]
pub struct WildMatch {
    pattern: Vec<char>,
}

impl WildMatch {
    /// Constructor with pattern which shall be used for matching.
    pub fn new(pattern: &str) -> WildMatch {
        let mut simplified: Vec<char> = Vec::new();
        let mut prev_was_star = false;
        for i in pattern.chars() {
            if i == '*' {
                if !prev_was_star {
                    simplified.push(i);
                }
                prev_was_star = true;
            } else {
                simplified.push(i);
                prev_was_star = false;
            }
        }
        WildMatch {
            pattern: simplified,
        }
    }
    /// Indicates whether the matcher finds a match in the input string.
    pub fn is_match(&self, input: &str) -> bool {
        if self.pattern.len() == 1 && self.pattern[0] == '*' {
            return true;
        }
        let mut pattern_idx = 0;
        let mut wildcard = false;
        for input_char in input.chars() {
            let pattern_char = self.pattern.get(pattern_idx);
            if pattern_char.is_none() {
                return true;
            }
            let pattern_char = pattern_char.unwrap();
            if pattern_char == &input_char || pattern_char == &'?' {
                pattern_idx += 1;
                if wildcard {
                    wildcard = false;
                }
            } else if wildcard {
                continue;
            } else if pattern_char == &'*' {
                wildcard = true;
                pattern_idx += 1;
            } else {
                pattern_idx = 0;
            }
        }
        return self.pattern.get(pattern_idx).is_none();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    #[test_case("**", test_name = "star_star")]
    #[test_case("*", test_name = "star")]
    #[test_case("c?*", test_name = "c_q_star")]
    #[test_case("???", test_name = "qqq")]
    #[test_case("c?t", test_name = "c_q_t")]
    #[test_case("cat", test_name = "cat")]
    fn valid_match_cat(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.is_match("cat"));
    }

    #[test_case("*d*", test_name = "star_d_star")]
    #[test_case("*d", test_name = "star_d")]
    #[test_case("d*", test_name = "d_star")]
    #[test_case("????", test_name = "q4")]
    #[test_case("?????", test_name = "q5")]
    #[test_case("*????", test_name = "wild_q_four")]
    #[test_case("cats", test_name = "longer")]
    #[test_case("cat?", test_name = "longer_q")]
    fn no_match_cat(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert!(!m.is_match("cat"));
    }

    #[test]
    fn longer_string_match() {
        let m = WildMatch::new("*cat*");
        assert!(m.is_match("d&(*og_cat_dog"));
    }
}
