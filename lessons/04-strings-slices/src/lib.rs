//! Lesson 04 — strings and slices. Replace every `todo!()` until `cargo test` passes.

/// Length in bytes (UTF-8 code units), like PHP's `strlen`. `"héllo"` → 6.
pub fn byte_len(s: &str) -> usize {
    todo!()
}

/// Number of `char`s (Unicode scalar values), like PHP's `mb_strlen`. `"héllo"` → 5.
/// Not what a reader counts as characters: `"👍🏽"` (thumbs up + skin tone) is 2 chars.
pub fn char_count(s: &str) -> usize {
    todo!()
}

/// The first word of `s`, borrowed from `s` (no copy).
/// Words are separated by any Unicode whitespace (`char::is_whitespace`); leading whitespace
/// is skipped and punctuation is part of the word: `"hello, world"` → `"hello,"`.
/// `""` if `s` has no word.
pub fn first_word(s: &str) -> &str {
    todo!()
}

/// The first `max` chars of `s`, borrowed from `s`. `s` itself if it has `max` chars or fewer.
/// `("héllo", 2)` → `"hé"`.
pub fn truncate_chars(s: &str, max: usize) -> &str {
    todo!()
}

/// The longest prefix of `s` that is at most `max_bytes` bytes long without splitting a char,
/// like PHP's `mb_strcut($s, 0, $max_bytes)`. `s` itself if `max_bytes >= s.len()`.
/// `("héllo", 2)` → `"h"`: `é` occupies bytes 1..3, so cutting at 2 would split it.
pub fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    todo!()
}

/// Parses `"key=value"`, splitting at the first `=`. Both parts are trimmed and borrowed from `s`.
/// `None` if `s` has no `=` or the trimmed key is empty. The value may be empty or contain `=`:
/// `" q = a=b "` → `Some(("q", "a=b"))`.
pub fn parse_pair(s: &str) -> Option<(&str, &str)> {
    todo!()
}

/// `s` with its chars in reverse order: `"héllo"` → `"olléh"`.
/// Reverses chars, not what a reader sees: a combining accent (U+0301) that followed its
/// letter ends up in front of it.
pub fn reverse_chars(s: &str) -> String {
    todo!()
}

/// Every whitespace-separated word with its first char uppercased (`char::to_uppercase`) and
/// the remaining chars lowercased, joined by single spaces. Leading, trailing, and repeated
/// whitespace is dropped. One char can uppercase to several: `"ßa"` → `"SSa"`.
pub fn title_case(s: &str) -> String {
    todo!()
}

/// `true` if `s` reads the same forwards and backwards, considering only alphanumeric chars
/// (`char::is_alphanumeric`) and ignoring case (`char::to_lowercase`).
/// A string without alphanumeric chars, including `""`, is a palindrome.
pub fn is_palindrome(s: &str) -> bool {
    todo!()
}

/// Sum of all values; `0` for an empty slice. Each value is widened to `i64` before adding,
/// so `[i32::MAX, 1]` sums to `2147483648` instead of overflowing.
pub fn sum(values: &[i32]) -> i64 {
    todo!()
}

/// The largest sum of two neighbouring values, widened to `i64` like `sum`.
/// `None` for fewer than two values. `[1, 5, -2, 4]` → `Some(6)`.
pub fn max_adjacent_sum(values: &[i32]) -> Option<i64> {
    todo!()
}

/// The part of `values` from its first to its last non-zero value, borrowed from `values`.
/// Zeros in between are kept: `[0, 0, 3, 0, 4, 0]` → `[3, 0, 4]`.
/// An empty slice if `values` has no non-zero value.
pub fn trim_zeros(values: &[i32]) -> &[i32] {
    todo!()
}

/// Describes `values` by its shape:
/// `[]` → `"empty"`, `[7]` → `"one: 7"`, `[1, 2]` → `"two: 1, 2"`, and three or more values
/// → `"many: 1 to 9 (5 values)"` (first value, last value, count).
pub fn summarize(values: &[i32]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_bytes() {
        assert_eq!(byte_len(""), 0);
        assert_eq!(byte_len("hello"), 5);
        assert_eq!(byte_len("héllo"), 6);
        assert_eq!(byte_len("日本語"), 9);
        assert_eq!(byte_len("🦀"), 4);
    }

    #[test]
    fn counts_chars() {
        assert_eq!(char_count(""), 0);
        assert_eq!(char_count("hello"), 5);
        assert_eq!(char_count("héllo"), 5);
        assert_eq!(char_count("日本語"), 3);
        assert_eq!(char_count("🦀"), 1);
        assert_eq!(char_count("👍🏽"), 2);
    }

    // Passes from the start: it shows the panic the truncate functions must avoid.
    #[test]
    #[should_panic(expected = "is not a char boundary")]
    fn byte_slicing_inside_a_char_panics() {
        let s = String::from("héllo");
        let _ = &s[0..2];
    }

    #[test]
    fn finds_first_word() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("hello, world"), "hello,");
        assert_eq!(first_word("   indented text"), "indented");
        assert_eq!(first_word("single"), "single");
        assert_eq!(first_word("tab\tseparated"), "tab");
        assert_eq!(first_word("héllo wörld"), "héllo");
        assert_eq!(first_word("日本語\u{3000}テキスト"), "日本語");
    }

    #[test]
    fn first_word_of_blank_input_is_empty() {
        assert_eq!(first_word(""), "");
        assert_eq!(first_word("   \n\t"), "");
    }

    #[test]
    fn first_word_borrows_from_the_input() {
        let owned = String::from("hello world");
        let word = first_word(&owned);
        assert_eq!(word, "hello");
        assert_eq!(word.as_ptr(), owned.as_ptr());
        assert_eq!(first_word(&owned[6..]), "world");
    }

    #[test]
    fn truncates_to_whole_chars() {
        assert_eq!(truncate_chars("hello", 2), "he");
        assert_eq!(truncate_chars("héllo", 2), "hé");
        assert_eq!(truncate_chars("日本語", 2), "日本");
        assert_eq!(truncate_chars("🦀rust", 1), "🦀");
    }

    #[test]
    fn truncate_chars_edge_cases() {
        assert_eq!(truncate_chars("", 3), "");
        assert_eq!(truncate_chars("abc", 0), "");
        assert_eq!(truncate_chars("abc", 3), "abc");
        assert_eq!(truncate_chars("héllo", 99), "héllo");
    }

    #[test]
    fn truncates_to_a_byte_budget_without_splitting_chars() {
        assert_eq!(truncate_bytes("hello", 3), "hel");
        assert_eq!(truncate_bytes("héllo", 2), "h");
        assert_eq!(truncate_bytes("héllo", 3), "hé");
        assert_eq!(truncate_bytes("日本語", 4), "日");
        assert_eq!(truncate_bytes("日本語", 2), "");
        assert_eq!(truncate_bytes("🦀🦀", 7), "🦀");
    }

    #[test]
    fn truncate_bytes_edge_cases() {
        assert_eq!(truncate_bytes("", 5), "");
        assert_eq!(truncate_bytes("abc", 0), "");
        assert_eq!(truncate_bytes("日本語", 9), "日本語");
        assert_eq!(truncate_bytes("日本語", 100), "日本語");
    }

    #[test]
    fn parses_key_value_pairs() {
        assert_eq!(parse_pair("name=Ada"), Some(("name", "Ada")));
        assert_eq!(
            parse_pair("  name =  Ada Lovelace "),
            Some(("name", "Ada Lovelace"))
        );
        assert_eq!(parse_pair("city=Zürich"), Some(("city", "Zürich")));
    }

    #[test]
    fn splits_pairs_at_the_first_equals_sign() {
        assert_eq!(parse_pair(" q = a=b "), Some(("q", "a=b")));
        assert_eq!(parse_pair("flag="), Some(("flag", "")));
        assert_eq!(parse_pair("flag =  "), Some(("flag", "")));
    }

    #[test]
    fn rejects_pairs_without_a_key_or_equals_sign() {
        assert_eq!(parse_pair(""), None);
        assert_eq!(parse_pair("novalue"), None);
        assert_eq!(parse_pair("=value"), None);
        assert_eq!(parse_pair("   = value"), None);
    }

    #[test]
    fn reverses_chars() {
        assert_eq!(reverse_chars(""), "");
        assert_eq!(reverse_chars("hello"), "olleh");
        assert_eq!(reverse_chars("héllo"), "olléh");
        assert_eq!(reverse_chars("日本語"), "語本日");
        assert_eq!(reverse_chars("a🦀b"), "b🦀a");
    }

    #[test]
    fn reversing_chars_moves_combining_accents() {
        // "ne\u{301}" renders as "né": the accent is its own char.
        assert_eq!(reverse_chars("ne\u{301}"), "\u{301}en");
    }

    #[test]
    fn title_cases_each_word() {
        assert_eq!(title_case("hello world"), "Hello World");
        assert_eq!(title_case("hELLO wORLD"), "Hello World");
        assert_eq!(title_case("élan vital"), "Élan Vital");
        assert_eq!(title_case("日本 語"), "日本 語");
    }

    #[test]
    fn title_case_normalizes_whitespace() {
        assert_eq!(title_case("  many   spaces\there\n"), "Many Spaces Here");
        assert_eq!(title_case(""), "");
        assert_eq!(title_case(" \t\n"), "");
    }

    #[test]
    fn title_case_can_change_length() {
        assert_eq!(title_case("STRASSE straße"), "Strasse Straße");
        assert_eq!(title_case("ßa"), "SSa");
    }

    #[test]
    fn detects_palindromes() {
        assert!(is_palindrome("racecar"));
        assert!(is_palindrome("A man, a plan, a canal: Panama!"));
        assert!(is_palindrome("No 'x' in Nixon"));
        assert!(is_palindrome("12321"));
        assert!(is_palindrome("Été"));
        assert!(is_palindrome("日本日"));
        assert!(is_palindrome("ab🦀ba"));
    }

    #[test]
    fn rejects_non_palindromes() {
        assert!(!is_palindrome("hello"));
        assert!(!is_palindrome("12 3"));
        assert!(!is_palindrome("Étés"));
        assert!(!is_palindrome("日本"));
        assert!(!is_palindrome("ab🦀ab"));
    }

    #[test]
    fn text_without_letters_or_digits_is_a_palindrome() {
        assert!(is_palindrome(""));
        assert!(is_palindrome("!?, "));
        assert!(is_palindrome("🦀"));
    }

    #[test]
    fn sums_any_slice() {
        let vec = vec![1, 2, 3, 4];
        let array = [10, 20];
        assert_eq!(sum(&vec), 10);
        assert_eq!(sum(&array), 30);
        assert_eq!(sum(&vec[1..3]), 5);
        assert_eq!(sum(&[-5, 5]), 0);
        assert_eq!(sum(&[]), 0);
    }

    #[test]
    fn sum_does_not_overflow() {
        assert_eq!(sum(&[i32::MAX, 1]), 2_147_483_648);
        assert_eq!(sum(&[i32::MIN, -1]), -2_147_483_649);
    }

    #[test]
    fn finds_max_adjacent_sum() {
        assert_eq!(max_adjacent_sum(&[1, 5, -2, 4]), Some(6));
        assert_eq!(max_adjacent_sum(&[-3, -1, -4]), Some(-4));
        assert_eq!(max_adjacent_sum(&[2, 2]), Some(4));
        assert_eq!(max_adjacent_sum(&[i32::MAX, i32::MAX]), Some(4_294_967_294));
    }

    #[test]
    fn max_adjacent_sum_needs_two_values() {
        assert_eq!(max_adjacent_sum(&[]), None);
        assert_eq!(max_adjacent_sum(&[7]), None);
    }

    #[test]
    fn trims_zeros_at_both_ends() {
        assert_eq!(trim_zeros(&[0, 0, 3, 0, 4, 0]), &[3, 0, 4]);
        assert_eq!(trim_zeros(&[1, 0, 2]), &[1, 0, 2]);
        assert_eq!(trim_zeros(&[0, -5]), &[-5]);
    }

    #[test]
    fn trim_zeros_of_only_zeros_is_empty() {
        assert!(trim_zeros(&[]).is_empty());
        assert!(trim_zeros(&[0, 0, 0]).is_empty());
    }

    #[test]
    fn trim_zeros_borrows_from_the_input() {
        let values = vec![0, 7, 8, 0];
        let trimmed = trim_zeros(&values);
        assert_eq!(trimmed, &[7, 8]);
        assert_eq!(trimmed.as_ptr(), values[1..].as_ptr());
    }

    #[test]
    fn summarizes_by_shape() {
        assert_eq!(summarize(&[]), "empty");
        assert_eq!(summarize(&[7]), "one: 7");
        assert_eq!(summarize(&[1, 2]), "two: 1, 2");
        assert_eq!(summarize(&[1, 5, 9]), "many: 1 to 9 (3 values)");
        assert_eq!(summarize(&[-3, 0, 0, 0, -1]), "many: -3 to -1 (5 values)");
    }
}
