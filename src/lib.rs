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
    pattern: Vec<char>,
    match_min_len: usize,
}

impl WildMatch {
    /// Constructor with pattern which shall be used for matching.
    pub fn new(pattern: &str) -> WildMatch {
        let mut simplified: Vec<char> = Vec::new();
        let mut prev_was_star = false;
        let mut match_min_len = 0;
        for i in pattern.chars() {
            if i == '*' {
                if !prev_was_star {
                    simplified.push(i);
                }
                prev_was_star = true;
            } else {
                simplified.push(i);
                prev_was_star = false;
                match_min_len += 1;
            }
        }
        WildMatch {
            pattern: simplified,
            match_min_len: match_min_len,
        }
    }
    /// Indicates whether the matcher finds a match in the input string.
    pub fn is_match(&self, input: &str) -> bool {
        if self.pattern.len() == 1 && self.pattern[0] == '*' {
            return true;
        }
        let mut pattern_idx = 0;
        let mut pattern_len = 0;
        let mut wildcard = false;
        for input_char in input.chars() {
            let pattern_char = self.pattern.get(pattern_idx);
            if pattern_char.is_none() && pattern_len >= self.match_min_len {
                return wildcard;
            }
            let pattern_char = pattern_char.unwrap();
            if pattern_char == &input_char || pattern_char == &'?' {
                pattern_idx += 1;
                pattern_len += 1;
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
                pattern_len = 0;
            }
        }
        return self.pattern.get(pattern_idx).is_none() && pattern_len >= self.match_min_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    #[test_case("**", test_name = "star_star")]
    #[test_case("*", test_name = "star")]
    #[test_case("*?*", test_name = "star_q_star")]
    #[test_case("c*", test_name = "c_star")]
    #[test_case("c?*", test_name = "c_q_star")]
    #[test_case("???", test_name = "qqq")]
    #[test_case("c?t", test_name = "c_q_t")]
    #[test_case("cat", test_name = "cat")]
    fn is_match(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.is_match("cat"));
    }

    #[test_case("*d*", test_name = "star_d_star")]
    #[test_case("*d", test_name = "star_d")]
    #[test_case("d*", test_name = "d_star")]
    #[test_case("*c", test_name = "star_c")]
    #[test_case("?", test_name = "q")]
    #[test_case("??", test_name = "q2")]
    #[test_case("????", test_name = "q4")]
    #[test_case("?????", test_name = "q5")]
    #[test_case("*????", test_name = "wild_q_four")]
    #[test_case("cats", test_name = "longer")]
    #[test_case("cat?", test_name = "longer_q")]
    #[test_case("cacat", test_name = "cacat")]
    fn no_match(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert!(!m.is_match("cat"));
    }

    #[test]
    fn longer_string_match() {
        let m = WildMatch::new("*cat*");
        assert!(m.is_match("d&(*og_cat_dog"));
    }
}
