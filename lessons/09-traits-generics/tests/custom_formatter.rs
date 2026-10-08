//! An integration test is a separate crate that sees only the library's public API,
//! like another Composer package implementing your interface.

use traits_generics::{Export, Report, ReportFormatter, Row, export};

struct Markdown;

impl ReportFormatter for Markdown {
    fn format(&self, report: &Report) -> String {
        let mut out = format!("# {}\n\n| label | count |\n|---|---:|\n", report.title);
        for row in &report.rows {
            out += &format!("| {} | {} |\n", row.label, row.count);
        }
        out
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }
}

/// Implements only the required method.
struct TitleOnly;

impl ReportFormatter for TitleOnly {
    fn format(&self, report: &Report) -> String {
        report.title.clone()
    }
}

fn sample() -> Report {
    Report {
        title: String::from("Fruit"),
        rows: vec![Row {
            label: String::from("apples"),
            count: 3,
        }],
    }
}

#[test]
fn downstream_formatter_inherits_default_methods() {
    assert_eq!(TitleOnly.file_extension(), "txt");
    assert_eq!(TitleOnly.file_name("notes"), "notes.txt");
}

#[test]
fn export_accepts_downstream_formatters() {
    assert_eq!(
        export(&Markdown, &sample(), "fruit"),
        Export {
            file_name: String::from("fruit.md"),
            contents: String::from("# Fruit\n\n| label | count |\n|---|---:|\n| apples | 3 |\n"),
        }
    );
    assert_eq!(export(&TitleOnly, &sample(), "fruit").contents, "Fruit");
}
