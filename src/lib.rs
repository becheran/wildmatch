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
    /// Constructor with pattern which can be used for matching.
    pub fn new(pattern: &str) -> WildMatch {
        let mut simplified: Vec<char> = Vec::new();
        let mut prev_was_star = false;
        let mut match_min_len = 0;
        for i in pattern.chars() {
            match i {
                '*' => {
                    if !prev_was_star {
                        simplified.push(i);
                    }
                    prev_was_star = true;
                }
                _ => {
                    simplified.push(i);
                    prev_was_star = false;
                    match_min_len += 1;
                }
            }
        }
        WildMatch {
            pattern: simplified,
            match_min_len: match_min_len,
        }
    }

    /// Indicates whether the matcher finds a match in the input string.
    pub fn is_match(&self, input: &str) -> bool {
        let mut wildcard = false;
        let input_chars: Vec<char> = input.chars().collect();

        let contains_star = self.pattern.contains(&'*');

        if self.pattern.len() == 1 && self.pattern[0] == '*' {
            return true;
        } else if self.pattern.len() == 0 || 
                ((self.pattern.len() < input_chars.len()) && !contains_star) {
            return false;
        }

        let mut pattern_idx = 0;
        let mut pattern_len = 0;
        for idx in 0..input_chars.len() {
            let input_char = input_chars.get(idx).unwrap();
            match self.pattern.get(pattern_idx) {
                None if pattern_len >= self.match_min_len => {
                    if !wildcard {
                        pattern_idx = 0;
                        pattern_len = 0;
                        continue;
                    } else {
                        return true;
                    }
                }
                Some(c) if c == input_char || c == &'?' => {
                    pattern_idx += 1;
                    pattern_len += 1;
                    if wildcard {
                        wildcard = false;
                    }
                }
                Some(_) if wildcard => {
                    continue;
                }
                Some(c) if c == &'*' => {
                    wildcard = true;
                    pattern_idx += 1;
                    if self.pattern.get(pattern_idx) == Some(&input_char)
                        || self.pattern.get(pattern_idx) == Some(&'?')
                    {
                        pattern_idx += 1;
                        pattern_len += 1;
                        if wildcard {
                            wildcard = false;
                        }
                    }
                }
                _ => {
                    return false;
                }
            }
        }
        let current = self.pattern.get(pattern_idx);
        return (current.is_none() || current == Some(&'*')) && pattern_len >= self.match_min_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::assert_false;
    use ntest::test_case;

    #[test_case("**", test_name = "star_star")]
    #[test_case("*", test_name = "star")]
    #[test_case("*?*", test_name = "star_q_star")]
    #[test_case("c*", test_name = "c_star")]
    #[test_case("c?*", test_name = "c_q_star")]
    #[test_case("???", test_name = "qqq")]
    #[test_case("c?t", test_name = "c_q_t")]
    #[test_case("cat", test_name = "cat")]
    #[test_case("*cat", test_name = "star_cat")]
    #[test_case("cat*", test_name = "cat_star")]
    fn is_match(pattern: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.is_match("cat"));
    }

    #[test_case("*d*", test_name = "star_d_star")]
    #[test_case("*d", test_name = "star_d")]
    #[test_case("d*", test_name = "d_star")]
    #[test_case("*c", test_name = "star_c")]
    #[test_case("?", test_name = "questionmark")]
    #[test_case("??", test_name = "q2")]
    #[test_case("????", test_name = "q4")]
    #[test_case("?????", test_name = "q5")]
    #[test_case("*????", test_name = "wild_q_four")]
    #[test_case("cats", test_name = "longer")]
    #[test_case("cat?", test_name = "longer_q")]
    #[test_case("cacat", test_name = "cacat")]
    #[test_case("cat*dog", test_name = "cat_star_dog")]
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
    fn match_long(pattern: &str, expected: &str) {
        let m = WildMatch::new(pattern);
        assert!(m.is_match(expected))
    }
}
