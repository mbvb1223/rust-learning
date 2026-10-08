//! Lesson 06 — collections and iteration. Replace every `todo!()` until `cargo test` passes.

use std::collections::{HashMap, HashSet};

/// Normalizes one token (a piece of text without whitespace):
///
/// 1. trim ASCII punctuation (`char::is_ascii_punctuation`) from both ends,
/// 2. lowercase it (Unicode-aware: `"ÉTÉ"` → `"été"`).
///
/// `None` if nothing is left. Inner punctuation stays (`"don't"`, `"e-mail"`), and non-ASCII
/// punctuation such as `«` or `—` is never trimmed.
///
/// `"Hello,"` → `Some("hello")`, `"(e.g.)"` → `Some("e.g")`, `"--"` → `None`.
pub fn normalize_word(token: &str) -> Option<String> {
    todo!()
}

/// Counts the words of `text`: the tokens of `str::split_whitespace`, each normalized with
/// [`normalize_word`]. Tokens that normalize to `None` are skipped.
///
/// Loop version: a `for` loop and the entry API.
pub fn word_frequencies(text: &str) -> HashMap<String, usize> {
    todo!()
}

/// The distinct words of `text`, using the same rules as [`word_frequencies`].
///
/// Loop version.
pub fn unique_words(text: &str) -> HashSet<String> {
    todo!()
}

/// Reading `text` left to right (same word rules as [`word_frequencies`]), the first word that
/// was already seen earlier. `None` if no word repeats.
///
/// `"a b c b a"` → `Some("b")`: the second `b` comes before the second `a`.
pub fn first_repeated_word(text: &str) -> Option<String> {
    todo!()
}

/// The `n` most frequent words as `(word, count)` pairs, ordered by count descending, then by
/// word ascending (`str` ordering compares bytes, so `"zoo"` comes before `"été"`).
///
/// Returns every pair, still ordered, if `freqs` has fewer than `n` entries. `n == 0` → empty.
///
/// Loop version: clone only the words you return.
pub fn top_n(freqs: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    todo!()
}

/// Same contract as [`word_frequencies`].
///
/// Iterator version: no `for`, `while` or `loop`.
pub fn word_frequencies_iter(text: &str) -> HashMap<String, usize> {
    todo!()
}

/// Same contract as [`unique_words`].
///
/// Iterator version: one chain ending in `collect`.
pub fn unique_words_iter(text: &str) -> HashSet<String> {
    todo!()
}

/// Same contract as [`top_n`].
///
/// Iterator version: no `for`, `while` or `loop`.
pub fn top_n_iter(freqs: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    todo!()
}

/// Adds every count in `other` to `total`; words that `total` doesn't have yet are inserted.
///
/// `other` is consumed: move its keys into `total` instead of cloning them.
pub fn merge_counts(total: &mut HashMap<String, usize>, other: HashMap<String, usize>) {
    todo!()
}

/// In place, replaces every word that is in `banned` (exact, case-sensitive match) with one `*`
/// per `char`: `"héllo"` → `"*****"`. Other words are untouched; the length and order of `words`
/// don't change.
pub fn mask_words(words: &mut [String], banned: &HashSet<String>) {
    todo!()
}

/// Sorts `words` by length in `char`s, shortest first. Words of equal length keep their original
/// relative order (a stable sort).
pub fn sort_by_length(words: &mut [String]) {
    todo!()
}

/// The words of `freqs` for which `keep(word, count)` returns `true`, sorted ascending
/// (`str` ordering).
pub fn words_where(
    freqs: &HashMap<String, usize>,
    keep: impl Fn(&str, usize) -> bool,
) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs
            .iter()
            .map(|&(word, count)| (word.to_string(), count))
            .collect()
    }

    fn ranked(pairs: &[(&str, usize)]) -> Vec<(String, usize)> {
        pairs
            .iter()
            .map(|&(word, count)| (word.to_string(), count))
            .collect()
    }

    fn strings(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    fn set(words: &[&str]) -> HashSet<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    fn word(text: &str) -> Option<String> {
        Some(text.to_string())
    }

    fn sample_freqs() -> HashMap<String, usize> {
        counts(&[("go", 1), ("été", 2), ("rust", 5), ("zoo", 2), ("apple", 2)])
    }

    const SAMPLES: [&str; 9] = [
        "",
        "  \n\t ",
        "-- !!! ...",
        "The cat; the DOG. The end!",
        "a b c b a",
        "Don't stop -- don't STOP!",
        "Été, été... «Bonjour» bonjour",
        "one\ttwo\nthree\r\nTWO  one ONE",
        "To be, or not to be: that is the question.",
    ];

    #[test]
    fn normalize_trims_ascii_punctuation_and_lowercases() {
        assert_eq!(normalize_word("Hello,"), word("hello"));
        assert_eq!(normalize_word("\"Rust!\""), word("rust"));
        assert_eq!(normalize_word("(e.g.)"), word("e.g"));
        assert_eq!(normalize_word("plain"), word("plain"));
    }

    #[test]
    fn normalize_keeps_inner_punctuation() {
        assert_eq!(normalize_word("don't"), word("don't"));
        assert_eq!(normalize_word("E-Mail"), word("e-mail"));
        assert_eq!(normalize_word("...rock'n'roll..."), word("rock'n'roll"));
    }

    #[test]
    fn normalize_lowercases_unicode() {
        assert_eq!(normalize_word("ÉTÉ"), word("été"));
        assert_eq!(normalize_word("Ωmega!"), word("ωmega"));
    }

    #[test]
    fn normalize_does_not_trim_non_ascii_punctuation() {
        assert_eq!(normalize_word("«Bonjour»"), word("«bonjour»"));
        assert_eq!(normalize_word("—"), word("—"));
    }

    #[test]
    fn normalize_returns_none_when_nothing_is_left() {
        assert_eq!(normalize_word(""), None);
        assert_eq!(normalize_word("--"), None);
        assert_eq!(normalize_word("...!?"), None);
    }

    #[test]
    fn counts_normalized_words() {
        assert_eq!(
            word_frequencies("The cat; the DOG. The end!"),
            counts(&[("the", 3), ("cat", 1), ("dog", 1), ("end", 1)])
        );
    }

    #[test]
    fn splits_on_any_whitespace() {
        assert_eq!(
            word_frequencies("one\ttwo\nthree\r\nTWO  one ONE"),
            counts(&[("one", 3), ("two", 2), ("three", 1)])
        );
    }

    #[test]
    fn skips_tokens_that_normalize_to_nothing() {
        assert_eq!(
            word_frequencies("wait -- what ?!"),
            counts(&[("wait", 1), ("what", 1)])
        );
    }

    #[test]
    fn text_without_words_has_no_frequencies() {
        assert!(word_frequencies("").is_empty());
        assert!(word_frequencies("  \n\t ").is_empty());
        assert!(word_frequencies("-- !!! ...").is_empty());
    }

    #[test]
    fn unique_words_ignore_case_and_punctuation() {
        assert_eq!(
            unique_words("Don't stop -- don't STOP!"),
            set(&["don't", "stop"])
        );
        assert!(unique_words("").is_empty());
    }

    #[test]
    fn finds_the_earliest_repeat() {
        assert_eq!(first_repeated_word("a b c b a"), word("b"));
        assert_eq!(first_repeated_word("Rust is fun. RUST!"), word("rust"));
        assert_eq!(first_repeated_word("Go go"), word("go"));
    }

    #[test]
    fn no_repeat_without_a_repeated_word() {
        assert_eq!(first_repeated_word("every word is unique"), None);
        assert_eq!(first_repeated_word("-- --"), None);
        assert_eq!(first_repeated_word(""), None);
    }

    #[test]
    fn top_n_orders_by_count_then_word() {
        assert_eq!(
            top_n(&sample_freqs(), 4),
            ranked(&[("rust", 5), ("apple", 2), ("zoo", 2), ("été", 2)])
        );
    }

    #[test]
    fn top_n_returns_every_pair_when_n_is_large() {
        assert_eq!(
            top_n(&sample_freqs(), 10),
            ranked(&[("rust", 5), ("apple", 2), ("zoo", 2), ("été", 2), ("go", 1)])
        );
    }

    #[test]
    fn top_n_of_zero_or_of_an_empty_map_is_empty() {
        assert!(top_n(&sample_freqs(), 0).is_empty());
        assert!(top_n(&HashMap::new(), 3).is_empty());
    }

    #[test]
    fn top_n_does_not_depend_on_hash_order() {
        // Each new HashMap gets random hash keys, so these maps can iterate in different orders.
        let expected = top_n(&sample_freqs(), 5);
        for _ in 0..20 {
            assert_eq!(top_n(&sample_freqs(), 5), expected);
        }
    }

    #[test]
    fn ranks_words_of_a_text() {
        let freqs = word_frequencies("To be, or not to be: that is the question.");
        assert_eq!(top_n(&freqs, 3), ranked(&[("be", 2), ("to", 2), ("is", 1)]));
    }

    #[test]
    fn word_frequencies_iter_counts_normalized_words() {
        assert_eq!(
            word_frequencies_iter("The cat; the DOG. The end!"),
            counts(&[("the", 3), ("cat", 1), ("dog", 1), ("end", 1)])
        );
        assert!(word_frequencies_iter("-- !!!").is_empty());
    }

    #[test]
    fn unique_words_iter_ignore_case_and_punctuation() {
        assert_eq!(
            unique_words_iter("Don't stop -- don't STOP!"),
            set(&["don't", "stop"])
        );
    }

    #[test]
    fn top_n_iter_orders_by_count_then_word() {
        assert_eq!(
            top_n_iter(&sample_freqs(), 4),
            ranked(&[("rust", 5), ("apple", 2), ("zoo", 2), ("été", 2)])
        );
        assert_eq!(top_n_iter(&sample_freqs(), 10).len(), 5);
        assert!(top_n_iter(&sample_freqs(), 0).is_empty());
    }

    #[test]
    fn iterator_versions_match_loop_versions() {
        for text in SAMPLES {
            let freqs = word_frequencies(text);
            assert_eq!(word_frequencies_iter(text), freqs, "text: {text:?}");
            assert_eq!(
                unique_words_iter(text),
                unique_words(text),
                "text: {text:?}"
            );
            for n in 0..=8 {
                assert_eq!(
                    top_n_iter(&freqs, n),
                    top_n(&freqs, n),
                    "text: {text:?}, n: {n}"
                );
            }
        }
    }

    #[test]
    fn merge_adds_counts_and_inserts_new_words() {
        let mut total = counts(&[("a", 1), ("b", 2)]);
        merge_counts(&mut total, counts(&[("b", 3), ("c", 1)]));
        assert_eq!(total, counts(&[("a", 1), ("b", 5), ("c", 1)]));
    }

    #[test]
    fn merge_with_empty_maps() {
        let mut total = HashMap::new();
        merge_counts(&mut total, counts(&[("a", 2)]));
        assert_eq!(total, counts(&[("a", 2)]));

        merge_counts(&mut total, HashMap::new());
        assert_eq!(total, counts(&[("a", 2)]));
    }

    #[test]
    fn merging_frequencies_of_two_texts_counts_both() {
        let mut total = word_frequencies("the cat");
        merge_counts(&mut total, word_frequencies("The dog"));
        assert_eq!(total, word_frequencies("the cat The dog"));
    }

    #[test]
    fn masks_banned_words_with_one_star_per_char() {
        let mut words = strings(&["darn", "it", "héllo", "darn"]);
        mask_words(&mut words, &set(&["darn", "héllo"]));
        assert_eq!(words, strings(&["****", "it", "*****", "****"]));
    }

    #[test]
    fn masking_is_case_sensitive() {
        let mut words = strings(&["Darn", "darn"]);
        mask_words(&mut words, &set(&["darn"]));
        assert_eq!(words, strings(&["Darn", "****"]));
    }

    #[test]
    fn masking_with_nothing_banned_changes_nothing() {
        let mut words = strings(&["keep", "these"]);
        mask_words(&mut words, &HashSet::new());
        assert_eq!(words, strings(&["keep", "these"]));

        let mut empty: Vec<String> = Vec::new();
        mask_words(&mut empty, &set(&["x"]));
        assert!(empty.is_empty());
    }

    #[test]
    fn sorts_by_char_count_not_bytes() {
        let mut words = strings(&["ccc", "bb", "a", "aa", "é", "b"]);
        sort_by_length(&mut words);
        assert_eq!(words, strings(&["a", "é", "b", "bb", "aa", "ccc"]));

        let mut empty: Vec<String> = Vec::new();
        sort_by_length(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn sort_by_length_is_stable() {
        // Enough equal-length words that an unstable sort would reorder them.
        let words: Vec<String> = (0..200)
            .map(|i| match i % 3 {
                0 => format!("{i:05}"),
                1 => format!("{i:03}"),
                _ => format!("{i:04}"),
            })
            .collect();
        let mut expected: Vec<String> = Vec::new();
        for width in [3, 4, 5] {
            for w in &words {
                if w.len() == width {
                    expected.push(w.clone());
                }
            }
        }

        let mut sorted = words.clone();
        sort_by_length(&mut sorted);
        assert_eq!(sorted, expected);
    }

    #[test]
    fn words_where_filters_with_the_closure_and_sorts() {
        let min = 2;
        assert_eq!(
            words_where(&sample_freqs(), |_, count| count >= min),
            strings(&["apple", "rust", "zoo", "été"])
        );
        assert_eq!(
            words_where(&sample_freqs(), |word, _| word.starts_with('r')),
            strings(&["rust"])
        );
    }

    #[test]
    fn words_where_can_return_nothing_or_everything() {
        assert!(words_where(&sample_freqs(), |_, _| false).is_empty());
        assert_eq!(
            words_where(&sample_freqs(), |_, _| true),
            strings(&["apple", "go", "rust", "zoo", "été"])
        );
        assert!(words_where(&HashMap::new(), |_, _| true).is_empty());
    }
}
