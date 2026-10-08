//! Part D — five cases that don't compile. Build with `--features broken`, fix each one
//! without changing the behaviour its doc comment describes, then make the tests pass.

// Case 1

/// The shorter of `a` and `b` by `char` count; on a tie, `a`.
pub fn shorter(a: &str, b: &str) -> &str {
    if b.chars().count() < a.chars().count() {
        b
    } else {
        a
    }
}

// Case 2

/// The first whitespace-separated word of `text`; `""` if there is none.
pub fn first_word(text: &str) -> &str {
    let owned = text.to_string();
    owned.split_whitespace().next().unwrap_or("")
}

// Case 3

/// The `char` count of whichever of `"Ada"` and `"Ada Lovelace"` [`shorter`] returns.
pub fn shorter_name_len() -> usize {
    let first = String::from("Ada");
    let result;
    {
        let full = format!("{first} Lovelace");
        result = shorter(&first, &full);
    }
    result.chars().count()
}

// Case 4

pub struct Setting<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

/// Parses `key=value`. Key and value are lowercased and trimmed. `None` if there is no `=`.
pub fn parse_setting(line: &str) -> Option<Setting<'_>> {
    let lower = line.to_lowercase();
    let (key, value) = lower.split_once('=')?;
    Some(Setting {
        key: key.trim(),
        value: value.trim(),
    })
}

// Case 5

/// Walks the whitespace-separated words of a borrowed text.
pub struct Cursor<'a> {
    rest: &'a str,
}

impl<'a> Cursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { rest: text }
    }

    /// The next word, advancing past it. `None` once the text is used up.
    pub fn next_word(&mut self) -> Option<&str> {
        let trimmed = self.rest.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let (word, rest) = trimmed.split_at(end);
        self.rest = rest;
        Some(word)
    }
}

/// Every whitespace-separated word of `text`, in order.
pub fn words(text: &str) -> Vec<&str> {
    let mut cursor = Cursor::new(text);
    let mut found = Vec::new();
    while let Some(word) = cursor.next_word() {
        found.push(word);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_1_shorter() {
        assert_eq!(shorter("fig", "apple"), "fig");
        assert_eq!(shorter("apple", "fig"), "fig");
        assert_eq!(shorter("same", "size"), "same");
    }

    #[test]
    fn case_2_first_word() {
        assert_eq!(first_word("  hello world"), "hello");
        assert_eq!(first_word("   "), "");
    }

    #[test]
    fn case_3_shorter_name_len() {
        assert_eq!(shorter_name_len(), 3);
    }

    #[test]
    fn case_4_parse_setting() {
        let setting = parse_setting(" Theme = DARK ").unwrap();
        assert_eq!(setting.key, "theme");
        assert_eq!(setting.value, "dark");
        assert!(parse_setting("no equals sign").is_none());
    }

    #[test]
    fn case_5_words() {
        assert_eq!(
            words(" alpha  beta\tgamma "),
            vec!["alpha", "beta", "gamma"]
        );
        assert!(words("   ").is_empty());
    }
}
