use regex::Regex;
use std::sync::OnceLock;

static WORD_RE: OnceLock<Regex> = OnceLock::new();

fn word_re() -> &'static Regex {
    WORD_RE.get_or_init(|| Regex::new(r"[A-Za-z][A-Za-z0-9]*|[0-9]+").unwrap())
}

/// Split camelCase/PascalCase/ALL_CAPS word into lowercase tokens.
fn split_camel(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let mut tokens: Vec<String> = Vec::new();
    let mut start = 0;

    while start < n {
        let c = chars[start];

        if c.is_ascii_digit() {
            let end = chars[start..]
                .iter()
                .position(|&x| !x.is_ascii_digit())
                .map(|p| start + p)
                .unwrap_or(n);
            tokens.push(chars[start..end].iter().collect());
            start = end;
            continue;
        }

        if c.is_uppercase() {
            // Count consecutive uppercase chars
            let upper_run = chars[start..]
                .iter()
                .take_while(|&&x| x.is_uppercase())
                .count();

            if upper_run == 1 {
                // Single uppercase: take it + following lowercase/digit run
                let lower_end = chars[start + 1..]
                    .iter()
                    .position(|&x| !x.is_lowercase() && !x.is_ascii_digit())
                    .map(|p| start + 1 + p)
                    .unwrap_or(n);
                tokens.push(chars[start..lower_end].iter().collect());
                start = lower_end;
            } else {
                // Multiple uppercase (e.g. HTML, ABCDef)
                // If next after uppercase run is lowercase, last uppercase starts new word
                let end = if start + upper_run < n && chars[start + upper_run].is_lowercase() {
                    start + upper_run - 1
                } else {
                    start + upper_run
                };
                let segment: String = chars[start..end].iter().collect();
                if !segment.is_empty() {
                    tokens.push(segment);
                }
                start = end;
            }
            continue;
        }

        if c.is_lowercase() {
            let end = chars[start..]
                .iter()
                .position(|&x| !x.is_lowercase() && !x.is_ascii_digit())
                .map(|p| start + p)
                .unwrap_or(n);
            tokens.push(chars[start..end].iter().collect());
            start = end;
            continue;
        }

        start += 1;
    }

    tokens
}

pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for word in word_re().find_iter(text) {
        for t in split_camel(word.as_str()) {
            let lower = t.to_lowercase();
            if lower.len() > 2 {
                tokens.push(lower);
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(tokenize(""), Vec::<String>::new());
    }

    #[test]
    fn simple_lowercase_word() {
        assert_eq!(tokenize("hello"), vec!["hello"]);
    }

    #[test]
    fn camel_case_split() {
        assert_eq!(tokenize("camelCase"), vec!["camel", "case"]);
    }

    #[test]
    fn pascal_case_split() {
        assert_eq!(tokenize("PascalCase"), vec!["pascal", "case"]);
    }

    #[test]
    fn snake_case_split() {
        // underscores são separadores para word_re
        assert_eq!(tokenize("snake_case"), vec!["snake", "case"]);
    }

    #[test]
    fn screaming_snake_split() {
        assert_eq!(tokenize("SCREAMING_SNAKE"), vec!["screaming", "snake"]);
    }

    #[test]
    fn acronym_before_word() {
        // HTTPServer → ["http", "server"]
        let result = tokenize("HTTPServer");
        assert_eq!(result, vec!["http", "server"]);
    }

    #[test]
    fn short_tokens_filtered() {
        // "a" (len=1) e "to" (len=2) são filtrados; min é len > 2
        let result = tokenize("a to fox");
        assert_eq!(result, vec!["fox"]);
    }

    #[test]
    fn three_char_token_kept() {
        // "the" tem len=3 > 2, deve ser mantido (sem lista de stopwords)
        assert!(tokenize("the fox").contains(&"the".to_string()));
    }

    #[test]
    fn digits_in_lowercase_run() {
        // dígitos dentro de run lowercase ficam junto com a palavra
        let result = tokenize("address123");
        assert_eq!(result, vec!["address123"]);
    }

    #[test]
    fn standalone_number() {
        // número puro de 3+ dígitos é mantido
        assert!(tokenize("version 123").contains(&"123".to_string()));
    }

    #[test]
    fn punctuation_ignored() {
        let result = tokenize("foo.bar(baz)");
        assert_eq!(result, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn multiple_words_in_sentence() {
        let result = tokenize("authenticate user session");
        assert_eq!(result, vec!["authenticate", "user", "session"]);
    }
}
