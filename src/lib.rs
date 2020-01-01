#[derive(Debug)]
pub struct WildMatch {
    pattern: Vec<char>,
}

impl WildMatch {
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
    pub fn is_match(&self, input: &str) -> bool {
        if self.pattern.len() == 1 && self.pattern[0] == '*' {
            true
        } else {
            let mut pattern_idx = 0;
            let mut wildcard = false;
            for input_char in input.chars() {
                let pattern_char = self.pattern.get(pattern_idx);
                if pattern_char.is_none() {
                    return true;
                }
                let pattern_char = pattern_char.unwrap();
                println!("inp {}", input_char);
                println!("pat {}", pattern_char);
                if pattern_char == &input_char || pattern_char == &'?' {
                    println!("MATCH");
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
                    println!("RESET");
                    pattern_idx = 0;
                }
            }
            return self.pattern.get(pattern_idx).is_none();
        }
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
}
