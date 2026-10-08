//! Headings → a nested Markdown list of links.

use crate::markdown::Heading;

/// Renders `headings` as a Markdown table of contents.
///
/// - Anchors come from one [`crate::markdown::slug::Slugger`] fed with *every* heading in
///   order, including headings deeper than `max_level`, so they match the anchors GitHub
///   generates for the whole document.
/// - Only headings with `level <= max_level` are listed, in order.
/// - Each listed heading is one line: `"{indent}- [{title}](#{anchor})\n"`, title unchanged.
/// - The indent is two spaces per level below the shallowest *listed* level. If that is `##`,
///   a `##` heading gets no indent and a `####` heading gets four spaces.
/// - Nothing listed → `""`.
///
/// # Examples
///
/// ```
/// use modules_testing::Heading;
/// use modules_testing::toc::render;
///
/// let headings = vec![
///     Heading { level: 2, title: "Setup".to_string() },
///     Heading { level: 3, title: "Docker".to_string() },
/// ];
/// assert_eq!(render(&headings, 6), "- [Setup](#setup)\n  - [Docker](#docker)\n");
/// ```
pub fn render(headings: &[Heading], max_level: u8) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headings(list: &[(u8, &str)]) -> Vec<Heading> {
        list.iter()
            .map(|&(level, title)| Heading {
                level,
                title: title.to_string(),
            })
            .collect()
    }

    #[test]
    fn renders_a_nested_list() {
        let input = headings(&[(1, "Title"), (2, "Install"), (3, "Docker"), (2, "Usage")]);
        assert_eq!(
            render(&input, 6),
            "- [Title](#title)\n  - [Install](#install)\n    - [Docker](#docker)\n  - [Usage](#usage)\n"
        );
    }

    #[test]
    fn indents_relative_to_the_shallowest_listed_level() {
        let input = headings(&[(2, "A"), (4, "B"), (2, "C")]);
        assert_eq!(render(&input, 6), "- [A](#a)\n    - [B](#b)\n- [C](#c)\n");

        let input = headings(&[(3, "Late start"), (1, "Top")]);
        assert_eq!(
            render(&input, 6),
            "    - [Late start](#late-start)\n- [Top](#top)\n"
        );
    }

    #[test]
    fn max_level_filters_deeper_headings() {
        let input = headings(&[(1, "Title"), (2, "Install"), (3, "Docker"), (2, "Usage")]);
        assert_eq!(
            render(&input, 2),
            "- [Title](#title)\n  - [Install](#install)\n  - [Usage](#usage)\n"
        );
        assert_eq!(render(&input, 1), "- [Title](#title)\n");
        assert_eq!(render(&headings(&[(3, "Deep")]), 2), "");
        assert_eq!(render(&[], 6), "");
    }

    #[test]
    fn anchors_count_unlisted_headings() {
        let input = headings(&[(1, "Notes"), (3, "Usage"), (2, "Usage")]);
        assert_eq!(
            render(&input, 2),
            "- [Notes](#notes)\n  - [Usage](#usage-1)\n"
        );
    }

    #[test]
    fn keeps_titles_unchanged() {
        let input = headings(&[(2, "Why `Option<T>`?")]);
        assert_eq!(render(&input, 6), "- [Why `Option<T>`?](#why-optiont)\n");
    }
}
