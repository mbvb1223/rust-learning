//! Markdown text → headings.
//!
//! `fence` is a private child module and `MAX_LEVEL` is `pub(crate)`. Code outside this
//! crate — `main.rs`, `tests/`, doc tests — can't name either:
//!
//! ```compile_fail
//! modules_testing::markdown::fence::is_fence("~~~");
//! ```
//!
//! ```compile_fail
//! let _ = modules_testing::markdown::MAX_LEVEL;
//! ```

mod fence;
pub mod slug;

/// Deepest ATX heading level (`######`).
pub(crate) const MAX_LEVEL: u8 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub title: String,
}

/// Parses one line as an ATX heading.
///
/// A heading is 1 to 6 (`MAX_LEVEL`) `#` characters at the very start of the line, then at least
/// one space or tab, then the title. The title is the rest of the line with surrounding
/// whitespace trimmed, and must not be empty.
///
/// Returns `None` for anything else: leading whitespace (`" # Title"`), 7 or more `#`,
/// no space or tab after the `#`s (`"#hashtag"`), or an empty title (`"#"`, `"## "`).
///
/// Simpler than CommonMark: no indentation allowed, and closing `#`s stay in the title
/// (`"## Title ##"` → `"Title ##"`).
///
/// # Examples
///
/// ```
/// use modules_testing::markdown::{Heading, parse_heading};
///
/// assert_eq!(
///     parse_heading("## Getting started"),
///     Some(Heading { level: 2, title: "Getting started".to_string() }),
/// );
/// assert_eq!(parse_heading("#hashtag"), None);
/// ```
pub fn parse_heading(line: &str) -> Option<Heading> {
    todo!()
}

/// Every heading in `markdown`, in document order.
///
/// Lines inside fenced code blocks are skipped. A fence line (three backticks or three tildes,
/// see the private `fence` module) opens a block, the next fence line of either kind closes it,
/// and fence lines are never headings themselves. An unclosed fence hides the rest of the
/// document.
pub fn headings(markdown: &str) -> Vec<Heading> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(level: u8, title: &str) -> Heading {
        Heading {
            level,
            title: title.to_string(),
        }
    }

    #[test]
    fn parses_every_level() {
        for level in 1..=MAX_LEVEL {
            let line = format!("{} Title", "#".repeat(usize::from(level)));
            assert_eq!(
                parse_heading(&line),
                Some(heading(level, "Title")),
                "{line}"
            );
        }
    }

    #[test]
    fn trims_the_title() {
        assert_eq!(
            parse_heading("#   Spaced out  "),
            Some(heading(1, "Spaced out"))
        );
        assert_eq!(parse_heading("##\tTabbed"), Some(heading(2, "Tabbed")));
        assert_eq!(parse_heading("## Title ##"), Some(heading(2, "Title ##")));
    }

    #[test]
    fn rejects_lines_that_are_not_headings() {
        for line in [
            "",
            "plain text",
            "#hashtag",
            "####### seven",
            " # indented",
            "#",
            "## ",
            "text # not at the start",
        ] {
            assert_eq!(parse_heading(line), None, "{line:?}");
        }
    }

    #[test]
    fn collects_headings_in_document_order() {
        let markdown = "# One\ntext\n## Two\n\n### Three\n## Four\n";
        assert_eq!(
            headings(markdown),
            vec![
                heading(1, "One"),
                heading(2, "Two"),
                heading(3, "Three"),
                heading(2, "Four"),
            ]
        );
        assert!(headings("").is_empty());
    }

    #[test]
    fn skips_fenced_code_blocks() {
        let markdown =
            "# Before\n```bash\n# comment\n```\n## Between\n~~~\n## hidden\n~~~\n## After";
        assert_eq!(
            headings(markdown),
            vec![
                heading(1, "Before"),
                heading(2, "Between"),
                heading(2, "After")
            ]
        );
    }

    #[test]
    fn unclosed_fence_hides_the_rest() {
        assert_eq!(
            headings("# Kept\n```\n# Hidden\n## Also hidden"),
            vec![heading(1, "Kept")]
        );
    }
}
