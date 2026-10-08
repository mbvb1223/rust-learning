//! Lesson 08 — modules and tests. A Markdown table-of-contents generator.
//! Replace every `todo!()` in `src/` and `tests/` until `cargo test` passes.
//!
//! Module tree:
//!
//! ```text
//! modules_testing          src/lib.rs
//! ├── cli                  src/cli.rs              arguments → Options (no I/O)
//! ├── markdown             src/markdown.rs         Markdown text → headings
//! │   ├── fence  (private) src/markdown/fence.rs
//! │   └── slug             src/markdown/slug.rs    heading title → anchor
//! └── toc                  src/toc.rs              headings → Markdown list
//! ```
//!
//! `src/main.rs` is a separate crate that uses this library through its public paths only.

pub mod cli;
pub mod markdown;
pub mod toc;

pub use markdown::Heading;

/// The table of contents of a whole Markdown document: [`markdown::headings`], then
/// [`toc::render`] with `max_level`.
///
/// # Examples
///
/// ```
/// let markdown = "# Title\n\n## Install\n\n~~~sh\n# a comment, not a heading\n~~~\n\n## Usage\n";
///
/// assert_eq!(
///     modules_testing::generate_toc(markdown, 6),
///     "- [Title](#title)\n  - [Install](#install)\n  - [Usage](#usage)\n",
/// );
/// ```
pub fn generate_toc(markdown: &str, max_level: u8) -> String {
    todo!()
}
