//! Lesson 07 — practical lifetimes. Replace every `todo!()` until `cargo test` passes,
//! then fix `src/broken.rs` until `cargo test --features broken` passes.

#[cfg(feature = "broken")]
pub mod broken;

/// The longest word in `text`, borrowed from `text`.
///
/// A word is a whitespace-separated token (`str::split_whitespace`) with leading and
/// trailing ASCII punctuation removed (`char::is_ascii_punctuation`). Tokens that end up
/// empty are skipped: `"Hello,"` → `"Hello"`, `"don't"` → `"don't"`, `"--"` → no word.
///
/// Length is counted in `char`s, not bytes. On a tie, the first word wins.
/// `None` if `text` has no words.
pub fn longest_word(text: &str) -> Option<&str> {
    todo!()
}

/// Whichever of `a` and `b` has more `char`s; on a tie, `a`.
pub fn longer_of<'a>(a: &'a str, b: &'a str) -> &'a str {
    todo!()
}

/// The part of `text` after the first occurrence of `marker`, with leading and trailing
/// whitespace trimmed. `None` if `marker` doesn't occur.
///
/// `text_after("Subject:  Hi ", ":")` → `Some("Hi")`; `text_after("a=b=c", "=")` → `Some("b=c")`.
/// An empty `marker` matches at the start, so the whole trimmed `text` is returned.
pub fn text_after<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    todo!()
}

/// The longest word across all `texts`, with the same word and length rules as
/// [`longest_word`]. On a tie, the word from the earlier text wins, then the earlier word
/// within that text. `None` if no text has a word.
pub fn longest_word_across<'a>(texts: &[&'a str]) -> Option<&'a str> {
    todo!()
}

/// `word` with leading and trailing ASCII punctuation removed, then lowercased with
/// `str::to_lowercase` (which handles non-ASCII letters).
/// `"Hello,"` → `"hello"`, `"ÉTÉ!"` → `"été"`. Empty if nothing is left.
pub fn normalize_word(word: &str) -> String {
    todo!()
}

/// One match of a needle inside a borrowed text, stored as byte offsets into that text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Highlight<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Highlight<'a> {
    /// The first occurrence of `needle` in `text`, case-sensitive.
    /// `None` if `needle` is empty or doesn't occur.
    pub fn find(text: &'a str, needle: &str) -> Option<Highlight<'a>> {
        todo!()
    }

    /// Every non-overlapping occurrence of `needle`, left to right: in `"aaaa"`, `"aa"`
    /// matches at byte offsets 0 and 2. Empty if `needle` is empty or doesn't occur.
    pub fn find_all(text: &'a str, needle: &str) -> Vec<Highlight<'a>> {
        todo!()
    }

    /// Byte offset where the match starts.
    pub fn start(&self) -> usize {
        todo!()
    }

    /// The whole text.
    pub fn text(&self) -> &str {
        todo!()
    }

    /// The matched part of the text.
    pub fn matched(&self) -> &'a str {
        todo!()
    }

    /// Everything before the match; `""` if the match starts the text.
    pub fn before(&self) -> &'a str {
        todo!()
    }

    /// Everything after the match; `""` if the match ends the text.
    pub fn after(&self) -> &'a str {
        todo!()
    }

    /// The whole text with the match wrapped in `open` and `close`:
    /// `"say hello world"` with `"[", "]"` → `"say [hello] world"`.
    pub fn render(&self, open: &str, close: &str) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starts(text: &str, needle: &str) -> Vec<usize> {
        Highlight::find_all(text, needle)
            .iter()
            .map(Highlight::start)
            .collect()
    }

    #[test]
    fn longest_word_finds_the_longest() {
        assert_eq!(longest_word("a bb ccc"), Some("ccc"));
        assert_eq!(longest_word("  spaced\tout\nwords  "), Some("spaced"));
        assert_eq!(longest_word("single"), Some("single"));
    }

    #[test]
    fn longest_word_first_wins_on_a_tie() {
        assert_eq!(longest_word("the quick brown fox"), Some("quick"));
        assert_eq!(longest_word("cat dog"), Some("cat"));
    }

    #[test]
    fn longest_word_strips_surrounding_punctuation() {
        assert_eq!(longest_word("Hi, world!!!"), Some("world"));
        assert_eq!(longest_word("(parenthesised) aside"), Some("parenthesised"));
        assert_eq!(longest_word("don't stop"), Some("don't"));
        assert_eq!(longest_word("-- ... wait"), Some("wait"));
    }

    #[test]
    fn longest_word_counts_chars_not_bytes() {
        assert_eq!(longest_word("éé abc"), Some("abc"));
        assert_eq!(longest_word("ab éé"), Some("ab"));
    }

    #[test]
    fn longest_word_is_none_without_words() {
        assert_eq!(longest_word(""), None);
        assert_eq!(longest_word("   \n\t "), None);
        assert_eq!(longest_word("-- ... !?"), None);
    }

    #[test]
    fn longer_of_picks_more_chars_and_a_on_a_tie() {
        assert_eq!(longer_of("apple", "fig"), "apple");
        assert_eq!(longer_of("fig", "apple"), "apple");
        assert_eq!(longer_of("same", "size"), "same");
        assert_eq!(longer_of("éé", "abc"), "abc");
        assert_eq!(longer_of("", ""), "");
    }

    #[test]
    fn text_after_returns_the_trimmed_rest() {
        assert_eq!(text_after("Subject:  Hi there ", ":"), Some("Hi there"));
        assert_eq!(text_after("a=b=c", "="), Some("b=c"));
        assert_eq!(text_after("key -> value", "->"), Some("value"));
        assert_eq!(text_after("ends with:", ":"), Some(""));
    }

    #[test]
    fn text_after_is_none_without_the_marker() {
        assert_eq!(text_after("no marker here", ":"), None);
        assert_eq!(text_after("", ":"), None);
    }

    #[test]
    fn text_after_with_empty_marker_returns_the_whole_text() {
        assert_eq!(text_after("  all of it ", ""), Some("all of it"));
    }

    #[test]
    fn text_after_result_does_not_borrow_the_marker() {
        let header = String::from("Content-Type: text/plain");
        let value = {
            let marker = String::from(":");
            text_after(&header, &marker)
        }; // `marker` is dropped here; `value` borrows only `header`
        assert_eq!(value, Some("text/plain"));
    }

    #[test]
    fn longest_word_across_searches_every_text() {
        assert_eq!(longest_word_across(&["a bb", "cccc d", "ee"]), Some("cccc"));
        assert_eq!(longest_word_across(&["", "  ", "word"]), Some("word"));
    }

    #[test]
    fn longest_word_across_prefers_the_earlier_text_then_the_earlier_word() {
        assert_eq!(longest_word_across(&["one two", "six ten"]), Some("one"));
        assert_eq!(longest_word_across(&["ab", "xyz uvw", "rst"]), Some("xyz"));
    }

    #[test]
    fn longest_word_across_is_none_without_words() {
        assert_eq!(longest_word_across(&[]), None);
        assert_eq!(longest_word_across(&["", "...", " "]), None);
    }

    #[test]
    fn longest_word_across_result_outlives_the_list() {
        let first = String::from("short words");
        let second = String::from("considerably longer");
        let longest = {
            let texts = vec![first.as_str(), second.as_str()];
            longest_word_across(&texts)
        }; // `texts` is dropped here; the result borrows the Strings, not the Vec
        assert_eq!(longest, Some("considerably"));
    }

    #[test]
    fn normalize_word_strips_punctuation_and_lowercases() {
        assert_eq!(normalize_word("Hello,"), "hello");
        assert_eq!(normalize_word("\"Quoted\""), "quoted");
        assert_eq!(normalize_word("Don't"), "don't");
        assert_eq!(normalize_word("ÉTÉ!"), "été");
        assert_eq!(normalize_word("..."), "");
    }

    #[test]
    fn normalize_word_turns_a_borrowed_result_into_an_owned_one() {
        let longest = longest_word("The QUICK, brown fox").map(normalize_word);
        assert_eq!(longest, Some(String::from("quick")));
    }

    #[test]
    fn highlight_finds_the_first_occurrence() {
        assert_eq!(Highlight::find("one two one", "one").unwrap().start(), 0);
        let hit = Highlight::find("say hello world", "hello").unwrap();
        assert_eq!(hit.start(), 4);
        assert_eq!(hit.matched(), "hello");
    }

    #[test]
    fn highlight_find_is_none_for_a_missing_or_empty_needle() {
        assert_eq!(Highlight::find("say hello", "bye"), None);
        assert_eq!(Highlight::find("say hello", "Hello"), None);
        assert_eq!(Highlight::find("say hello", ""), None);
        assert_eq!(Highlight::find("", "x"), None);
    }

    #[test]
    fn highlight_splits_the_text_around_the_match() {
        let hit = Highlight::find("say hello world", "hello").unwrap();
        assert_eq!(hit.before(), "say ");
        assert_eq!(hit.matched(), "hello");
        assert_eq!(hit.after(), " world");
        assert_eq!(hit.text(), "say hello world");
    }

    #[test]
    fn highlight_at_the_edges_has_empty_before_or_after() {
        let first = Highlight::find("hello world", "hello").unwrap();
        assert_eq!(first.before(), "");
        let last = Highlight::find("hello world", "world").unwrap();
        assert_eq!(last.after(), "");
        let whole = Highlight::find("hello", "hello").unwrap();
        assert_eq!((whole.before(), whole.after()), ("", ""));
    }

    #[test]
    fn highlight_offsets_are_bytes() {
        let hit = Highlight::find("café au lait", "au").unwrap();
        assert_eq!(hit.start(), 6);
        assert_eq!(hit.before(), "café ");
        assert_eq!(hit.after(), " lait");
    }

    #[test]
    fn highlight_slices_outlive_the_highlight() {
        let text = String::from("needle in a haystack");
        let (before, matched) = {
            let hit = Highlight::find(&text, "haystack").unwrap();
            (hit.before(), hit.matched())
        }; // `hit` is dropped here; the slices borrow `text`
        assert_eq!(before, "needle in a ");
        assert_eq!(matched, "haystack");
    }

    #[test]
    fn highlight_does_not_borrow_the_needle() {
        let text = String::from("needle in a haystack");
        // The formatted needle is a temporary, dropped at the end of this statement.
        let hit = Highlight::find(&text, &format!("hay{}", "stack")).unwrap();
        assert_eq!(hit.start(), 12);
    }

    #[test]
    fn find_all_returns_non_overlapping_matches_in_order() {
        assert_eq!(starts("one two one", "one"), vec![0, 8]);
        assert_eq!(starts("aaaa", "aa"), vec![0, 2]);
        assert_eq!(starts("aaa", "aa"), vec![0]);
    }

    #[test]
    fn find_all_is_empty_for_a_missing_or_empty_needle() {
        assert!(Highlight::find_all("one two", "three").is_empty());
        assert!(Highlight::find_all("one two", "").is_empty());
    }

    #[test]
    fn find_all_matches_borrow_the_text() {
        let text = String::from("to be or not to be");
        let hits = Highlight::find_all(&text, "be");
        let matched: Vec<&str> = hits.iter().map(Highlight::matched).collect();
        assert_eq!(matched, vec!["be", "be"]);
        assert_eq!(hits[1].before(), "to be or not to ");
    }

    #[test]
    fn render_wraps_the_match() {
        let hit = Highlight::find("say hello world", "hello").unwrap();
        assert_eq!(hit.render("[", "]"), "say [hello] world");
        assert_eq!(
            hit.render("<mark>", "</mark>"),
            "say <mark>hello</mark> world"
        );
        let whole = Highlight::find("hello", "hello").unwrap();
        assert_eq!(whole.render("*", "*"), "*hello*");
    }
}
