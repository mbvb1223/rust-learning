//! Heading title → link anchor, the way GitHub generates them.

use std::collections::HashSet;

/// The anchor GitHub generates for a heading title, without duplicate handling.
///
/// Lowercase the title with [`str::to_lowercase`], then map each `char`:
/// - alphanumeric ([`char::is_alphanumeric`]), `-` and `_` are kept
/// - a space `' '` becomes `-`
/// - anything else is dropped
///
/// Hyphens are never collapsed: in `"A — B"` the dash is dropped and both spaces stay, giving
/// `"a--b"`.
///
/// # Examples
///
/// ```
/// use modules_testing::markdown::slug::slugify;
///
/// assert_eq!(slugify("Run the tests"), "run-the-tests");
/// assert_eq!(slugify("Why `Option<T>`?"), "why-optiont");
/// ```
pub fn slugify(title: &str) -> String {
    todo!()
}

/// Hands out unique anchors in document order, like GitHub does for repeated headings.
#[derive(Debug, Default)]
pub struct Slugger {
    taken: HashSet<String>,
}

impl Slugger {
    /// A `Slugger` that hasn't returned any anchor yet.
    pub fn new() -> Self {
        todo!()
    }

    /// The [`slugify`] anchor of `title` if this `Slugger` hasn't returned it before. Otherwise
    /// the first of `"<slug>-1"`, `"<slug>-2"`, … that it hasn't returned before.
    ///
    /// # Examples
    ///
    /// ```
    /// use modules_testing::markdown::slug::Slugger;
    ///
    /// let mut slugger = Slugger::new();
    /// assert_eq!(slugger.unique("Usage"), "usage");
    /// assert_eq!(slugger.unique("Usage"), "usage-1");
    /// ```
    pub fn unique(&mut self, title: &str) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases_and_turns_spaces_into_hyphens() {
        assert_eq!(slugify("Run the Tests"), "run-the-tests");
        assert_eq!(slugify("CLI"), "cli");
    }

    #[test]
    fn drops_punctuation_without_collapsing_hyphens() {
        assert_eq!(
            slugify("Lesson 08 — Modules and tests"),
            "lesson-08--modules-and-tests"
        );
        assert_eq!(slugify("`cargo test` filtering"), "cargo-test-filtering");
        assert_eq!(slugify("Option<T> & Result"), "optiont--result");
        assert_eq!(slugify("Why?!"), "why");
        assert_eq!(slugify("!?"), "");
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn keeps_unicode_letters_digits_hyphens_and_underscores() {
        assert_eq!(slugify("Ünïcode Straße"), "ünïcode-straße");
        assert_eq!(
            slugify("snake_case vs kebab-case 2"),
            "snake_case-vs-kebab-case-2"
        );
    }

    #[test]
    fn repeated_titles_get_numeric_suffixes() {
        let mut slugger = Slugger::new();
        assert_eq!(slugger.unique("Intro"), "intro");
        assert_eq!(slugger.unique("Intro"), "intro-1");
        assert_eq!(slugger.unique("intro"), "intro-2");
        assert_eq!(slugger.unique("Other"), "other");
        assert_eq!(Slugger::new().unique("Intro"), "intro");
    }

    #[test]
    fn suffixes_skip_anchors_already_returned() {
        let mut slugger = Slugger::new();
        assert_eq!(slugger.unique("Intro 1"), "intro-1");
        assert_eq!(slugger.unique("Intro"), "intro");
        assert_eq!(slugger.unique("Intro"), "intro-2");
        assert_eq!(slugger.unique("Intro 1"), "intro-1-1");
    }
}
