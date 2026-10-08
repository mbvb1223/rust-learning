//! Lesson 09 — practical traits and generics. Replace every `todo!()` until `cargo test` passes.

use std::fmt::{self, Display};
use std::str::FromStr;

/// One report line: a label and its count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub label: String,
    pub count: usize,
}

/// `("apples", 3)` and `(String::from("apples"), 3)` both become `Row { label: "apples", count: 3 }`.
/// Implementing `From` also gives you `Into`: `let row: Row = ("apples", 3).into();`.
impl<L: Into<String>> From<(L, usize)> for Row {
    fn from((label, count): (L, usize)) -> Self {
        todo!()
    }
}

/// A titled list of rows. `Report::default()` has an empty title and no rows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    pub title: String,
    pub rows: Vec<Row>,
}

impl Report {
    /// A report with `title` (kept exactly as given) and no rows. Accepts `&str` or `String`.
    pub fn new(title: impl Into<String>) -> Self {
        todo!()
    }

    /// Appends `row` after the existing rows and returns the report, so calls chain:
    /// `Report::new("Fruit").with_row(("apples", 3)).with_row(("pears", 2))`.
    /// Accepts anything that converts into a `Row`, including a `Row`.
    pub fn with_row(mut self, row: impl Into<Row>) -> Self {
        todo!()
    }

    /// Sum of all counts; `0` when there are no rows.
    pub fn total(&self) -> usize {
        todo!()
    }

    /// The rows whose count is at least `min`, borrowed from the report, in their original order.
    pub fn rows_at_least(&self, min: usize) -> impl Iterator<Item = &Row> {
        // Bare `todo!()` doesn't compile here: `impl Trait` needs a concrete type. Replace the whole line.
        todo!() as std::iter::Empty<&Row>
    }
}

/// One-line summary: `"Fruit sales (3 rows, total 135)"`.
/// Says `"1 row"` for exactly one row and `"0 rows"` for none.
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Every item's `Display` output, in order, separated by `separator`; `""` for no items.
/// `join_display(&[1, 2, 3], ", ")` → `"1, 2, 3"`.
pub fn join_display(items: &[impl Display], separator: &str) -> String {
    todo!()
}

/// An output format a user can pick, e.g. with a `--format` flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FormatKind {
    #[default]
    Text,
    Csv,
}

impl FormatKind {
    pub const ALL: [FormatKind; 2] = [FormatKind::Text, FormatKind::Csv];
}

/// `"text"` or `"csv"`.
impl fmt::Display for FormatKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Parses `"text"` or `"csv"`, ignoring ASCII case (`"CSV"` works). Whitespace is not trimmed.
/// Anything else is a `ParseFormatError` holding the input unchanged.
impl FromStr for FormatKind {
    type Err = ParseFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

/// Same rules and errors as `FromStr`. Gives `FormatKind::try_from("CSV")` and `"csv".try_into()`.
impl TryFrom<&str> for FormatKind {
    type Error = ParseFormatError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// The input was not a known `FormatKind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFormatError {
    /// The rejected input, unchanged.
    pub input: String,
}

/// `unknown format "xml" (expected one of: text, csv)`: the input in `{:?}` form (quoted and
/// escaped), then every `FormatKind::ALL` value in order, separated by `", "`.
impl fmt::Display for ParseFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ParseFormatError {}

/// Turns a `Report` into the contents of one file.
pub trait ReportFormatter {
    /// The complete file contents for `report`.
    fn format(&self, report: &Report) -> String;

    /// File extension without the dot. Defaults to `"txt"`.
    fn file_extension(&self) -> &'static str {
        todo!()
    }

    /// `"{stem}.{extension}"` using `self.file_extension()`, e.g. `"sales.txt"`.
    /// A formatter that overrides the extension gets the matching file name for free.
    fn file_name(&self, stem: &str) -> String {
        todo!()
    }
}

/// An aligned plain-text table. `PlainText::default()` hides the total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlainText {
    /// Append a `total` line.
    pub show_total: bool,
}

impl ReportFormatter for PlainText {
    /// With `show_total: true`:
    ///
    /// ```text
    /// Fruit sales
    /// ===========
    /// apples          3
    /// bananas, ripe  12
    /// cherries      120
    /// total         135
    /// ```
    ///
    /// - Line 1 is the title; line 2 is `=` repeated once per `char` of the title.
    /// - Then one line per row, in order: the label left-aligned and padded with spaces to the
    ///   label width, one space, then the count right-aligned to the count width.
    /// - With `show_total`, a last line in the same layout, labelled `total`, holding `Report::total`.
    /// - Label width: the longest label, in `char`s. Count width: the most digits of any count.
    ///   Both widths include the `total` line when it is shown.
    /// - Every line, including the last, ends with `\n`.
    fn format(&self, report: &Report) -> String {
        todo!()
    }
}

/// RFC 4180 quoting for one CSV field. A field containing `delimiter`, `"`, `\r` or `\n` is
/// wrapped in double quotes, with every `"` inside doubled: `say "hi"` → `"say ""hi"""`.
/// Any other field is returned unchanged, including empty and space-padded ones.
pub fn csv_field(field: &str, delimiter: char) -> String {
    todo!()
}

/// CSV output options. The delimiter must not be `"`, `\r` or `\n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Csv {
    pub delimiter: char,
    /// Start with a header record.
    pub header: bool,
}

/// Comma delimiter, with a header.
impl Default for Csv {
    fn default() -> Self {
        todo!()
    }
}

impl ReportFormatter for Csv {
    /// With the default options:
    ///
    /// ```text
    /// label,count
    /// apples,3
    /// "bananas, ripe",12
    /// cherries,120
    /// ```
    ///
    /// - With `header`, a first record: `label`, the delimiter, `count`.
    /// - Then one record per row, in order: `csv_field(label, delimiter)`, the delimiter, the count.
    /// - Every record ends with `\n`. The title and total are not written.
    fn format(&self, report: &Report) -> String {
        todo!()
    }

    /// `"csv"`.
    fn file_extension(&self) -> &'static str {
        todo!()
    }
}

/// A rendered file, ready to write to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub file_name: String,
    pub contents: String,
}

/// Renders `report` with any formatter: `file_name` is `formatter.file_name(stem)`,
/// `contents` is `formatter.format(report)`.
pub fn export<F: ReportFormatter>(formatter: &F, report: &Report, stem: &str) -> Export {
    todo!()
}

/// `export` with the formatter that `kind` names: `PlainText::default()` or `Csv::default()`.
pub fn export_as(kind: FormatKind, report: &Report, stem: &str) -> Export {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn row(label: &str, count: usize) -> Row {
        Row {
            label: label.to_string(),
            count,
        }
    }

    fn report(title: &str, rows: Vec<Row>) -> Report {
        Report {
            title: title.to_string(),
            rows,
        }
    }

    fn fruit() -> Report {
        report(
            "Fruit sales",
            vec![
                row("apples", 3),
                row("bananas, ripe", 12),
                row("cherries", 120),
            ],
        )
    }

    #[test]
    fn row_from_tuple_accepts_str_and_string() {
        assert_eq!(Row::from(("apples", 3)), row("apples", 3));
        assert_eq!(Row::from((String::from("pears"), 2)), row("pears", 2));
        let converted: Row = ("kiwis", 0).into();
        assert_eq!(converted, row("kiwis", 0));
    }

    #[test]
    fn new_keeps_the_title_as_given() {
        assert_eq!(Report::new("Sales"), report("Sales", vec![]));
        assert_eq!(Report::new(String::from("  Sales ")).title, "  Sales ");
    }

    #[test]
    fn default_report_is_empty() {
        assert_eq!(Report::default(), Report::new(""));
    }

    #[test]
    fn with_row_appends_in_order() {
        let built = Report::new("Fruit sales")
            .with_row(("apples", 3))
            .with_row((String::from("bananas, ripe"), 12))
            .with_row(row("cherries", 120));
        assert_eq!(built, fruit());
    }

    #[test]
    fn total_sums_counts() {
        assert_eq!(fruit().total(), 135);
        assert_eq!(Report::default().total(), 0);
    }

    #[test]
    fn rows_at_least_filters_in_order() {
        let fruit = fruit();
        let labels: Vec<&str> = fruit
            .rows_at_least(10)
            .map(|row| row.label.as_str())
            .collect();
        assert_eq!(labels, ["bananas, ripe", "cherries"]);
        assert_eq!(fruit.rows_at_least(3).count(), 3);
        assert_eq!(fruit.rows_at_least(121).count(), 0);
    }

    #[test]
    fn displays_a_summary() {
        assert_eq!(fruit().to_string(), "Fruit sales (3 rows, total 135)");
        assert_eq!(
            format!("{}", report("One", vec![row("a", 7)])),
            "One (1 row, total 7)"
        );
        assert_eq!(report("None", vec![]).to_string(), "None (0 rows, total 0)");
    }

    #[test]
    fn join_display_accepts_any_display_type() {
        assert_eq!(join_display(&[1, 2, 3], ", "), "1, 2, 3");
        assert_eq!(join_display(&["solo"], ", "), "solo");
        assert_eq!(join_display(&[1.5, -2.0], " | "), "1.5 | -2");
        assert_eq!(join_display(&FormatKind::ALL, "/"), "text/csv");
        let none: &[i32] = &[];
        assert_eq!(join_display(none, ", "), "");
    }

    #[test]
    fn displays_format_kinds() {
        assert_eq!(FormatKind::Text.to_string(), "text");
        assert_eq!(FormatKind::Csv.to_string(), "csv");
    }

    #[test]
    fn parses_format_kinds_ignoring_case() {
        assert_eq!("text".parse::<FormatKind>(), Ok(FormatKind::Text));
        assert_eq!("csv".parse(), Ok(FormatKind::Csv));
        assert_eq!("CSV".parse(), Ok(FormatKind::Csv));
        assert_eq!(FormatKind::from_str("Text"), Ok(FormatKind::Text));
    }

    #[test]
    fn rejects_unknown_formats() {
        for input in ["xml", "", " csv", "txt"] {
            assert_eq!(
                input.parse::<FormatKind>(),
                Err(ParseFormatError {
                    input: input.to_string()
                })
            );
        }
    }

    #[test]
    fn try_from_str_follows_the_parse_rules() {
        assert_eq!(FormatKind::try_from("CSV"), Ok(FormatKind::Csv));
        assert_eq!(FormatKind::try_from("text"), Ok(FormatKind::Text));
        let rejected: Result<FormatKind, _> = "xml".try_into();
        assert_eq!(
            rejected,
            Err(ParseFormatError {
                input: String::from("xml")
            })
        );
    }

    #[test]
    fn parse_error_message_lists_valid_formats() {
        let error = ParseFormatError {
            input: String::from("xml"),
        };
        assert_eq!(
            error.to_string(),
            r#"unknown format "xml" (expected one of: text, csv)"#
        );
        let quoted = ParseFormatError {
            input: String::from(r#"a"b"#),
        };
        assert_eq!(
            quoted.to_string(),
            r#"unknown format "a\"b" (expected one of: text, csv)"#
        );
    }

    #[test]
    fn format_kinds_round_trip_through_strings() {
        for kind in FormatKind::ALL {
            assert_eq!(kind.to_string().parse(), Ok(kind));
        }
    }

    #[test]
    fn derived_default_and_hash_on_format_kind() {
        assert_eq!(FormatKind::default(), FormatKind::Text);
        let kinds: HashSet<FormatKind> = ["csv", "CSV", "text", "Text"]
            .into_iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(kinds, HashSet::from(FormatKind::ALL));
    }

    #[test]
    fn plain_text_uses_the_default_extension() {
        assert_eq!(PlainText::default().file_extension(), "txt");
        assert_eq!(PlainText::default().file_name("q3-sales"), "q3-sales.txt");
    }

    #[test]
    fn plain_text_aligns_rows() {
        assert_eq!(PlainText::default(), PlainText { show_total: false });
        let expected = "\
Fruit sales
===========
apples          3
bananas, ripe  12
cherries      120
";
        assert_eq!(PlainText::default().format(&fruit()), expected);
    }

    #[test]
    fn plain_text_total_line() {
        let expected = "\
Fruit sales
===========
apples          3
bananas, ripe  12
cherries      120
total         135
";
        assert_eq!(PlainText { show_total: true }.format(&fruit()), expected);
    }

    #[test]
    fn plain_text_total_line_counts_toward_widths() {
        let short = report("Short", vec![row("a", 5), row("b", 7)]);
        assert_eq!(
            PlainText { show_total: false }.format(&short),
            "Short\n=====\na 5\nb 7\n"
        );
        assert_eq!(
            PlainText { show_total: true }.format(&short),
            "Short\n=====\na      5\nb      7\ntotal 12\n"
        );
    }

    #[test]
    fn plain_text_measures_chars_not_bytes() {
        let menu = report("Café", vec![row("crème brûlée", 2), row("tea", 10)]);
        let expected = "\
Café
====
crème brûlée  2
tea          10
";
        assert_eq!(PlainText::default().format(&menu), expected);
    }

    #[test]
    fn plain_text_empty_report() {
        let empty = report("Empty", vec![]);
        assert_eq!(
            PlainText { show_total: false }.format(&empty),
            "Empty\n=====\n"
        );
        assert_eq!(
            PlainText { show_total: true }.format(&empty),
            "Empty\n=====\ntotal 0\n"
        );
    }

    #[test]
    fn csv_field_leaves_plain_fields_unchanged() {
        assert_eq!(csv_field("apples", ','), "apples");
        assert_eq!(csv_field("", ','), "");
        assert_eq!(csv_field(" padded ", ','), " padded ");
        assert_eq!(csv_field("a;b", ','), "a;b");
    }

    #[test]
    fn csv_field_quotes_special_characters() {
        assert_eq!(csv_field("bananas, ripe", ','), r#""bananas, ripe""#);
        assert_eq!(csv_field(r#"say "hi""#, ','), r#""say ""hi""""#);
        assert_eq!(csv_field(r#"""#, ','), r#""""""#);
        assert_eq!(csv_field("line 1\nline 2", ','), "\"line 1\nline 2\"");
        assert_eq!(csv_field("a\rb", ','), "\"a\rb\"");
    }

    #[test]
    fn csv_field_quotes_the_given_delimiter() {
        assert_eq!(csv_field("a;b", ';'), r#""a;b""#);
        assert_eq!(csv_field("a,b", ';'), "a,b");
        assert_eq!(csv_field("a\tb", '\t'), "\"a\tb\"");
    }

    #[test]
    fn csv_default_is_comma_with_header() {
        assert_eq!(
            Csv::default(),
            Csv {
                delimiter: ',',
                header: true
            }
        );
    }

    #[test]
    fn csv_writes_header_and_rows() {
        let expected = "label,count\napples,3\n\"bananas, ripe\",12\ncherries,120\n";
        assert_eq!(Csv::default().format(&fruit()), expected);
    }

    #[test]
    fn csv_follows_its_options() {
        let no_header = Csv {
            delimiter: ';',
            header: false,
        };
        assert_eq!(
            no_header.format(&fruit()),
            "apples;3\nbananas, ripe;12\ncherries;120\n"
        );
        let with_header = Csv {
            delimiter: ';',
            header: true,
        };
        assert_eq!(
            with_header.format(&report("x", vec![row("a;b", 1)])),
            "label;count\n\"a;b\";1\n"
        );
    }

    #[test]
    fn csv_empty_report() {
        let empty = report("Empty", vec![]);
        assert_eq!(Csv::default().format(&empty), "label,count\n");
        let no_header = Csv {
            header: false,
            ..Csv::default()
        };
        assert_eq!(no_header.format(&empty), "");
    }

    #[test]
    fn csv_overrides_the_extension() {
        assert_eq!(Csv::default().file_extension(), "csv");
        assert_eq!(Csv::default().file_name("q3-sales"), "q3-sales.csv");
    }

    #[test]
    fn export_works_with_any_formatter() {
        let fruit = fruit();
        let text = PlainText { show_total: true };
        assert_eq!(
            export(&text, &fruit, "fruit"),
            Export {
                file_name: String::from("fruit.txt"),
                contents: text.format(&fruit),
            }
        );
        let csv = Csv::default();
        assert_eq!(
            export(&csv, &fruit, "fruit"),
            Export {
                file_name: String::from("fruit.csv"),
                contents: csv.format(&fruit),
            }
        );
    }

    #[test]
    fn export_as_uses_the_default_formatter_for_a_kind() {
        let fruit = fruit();
        assert_eq!(
            export_as(FormatKind::Text, &fruit, "fruit"),
            export(&PlainText::default(), &fruit, "fruit")
        );
        assert_eq!(
            export_as(FormatKind::Csv, &fruit, "fruit"),
            export(&Csv::default(), &fruit, "fruit")
        );
    }
}
