//! Lesson 05 — errors and file I/O. Replace every `todo!()` here and in `src/main.rs` until `cargo test` passes.

use std::error::Error;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const USAGE: &str = "usage: errors-io [--max-width N] FILE";

pub const DEFAULT_MAX_WIDTH: usize = 80;

/// Everything that can go wrong in this program.
///
/// `io::Error` is neither `Clone` nor `PartialEq`, so this enum can't derive them either.
#[derive(Debug)]
pub enum AppError {
    /// Bad command-line arguments. Holds a message such as `"missing FILE argument"`.
    Usage(String),
    /// The input file doesn't exist.
    NotFound(PathBuf),
    /// The input file exists but its contents aren't valid UTF-8.
    InvalidUtf8(PathBuf),
    /// The input file has no words: it is empty or whitespace only.
    EmptyInput(PathBuf),
    /// Any other I/O failure: permission denied, a directory instead of a file, a closed stdout, …
    Io(io::Error),
}

impl AppError {
    /// Process exit code: `2` for `Usage`, `1` for every other variant.
    pub fn exit_code(&self) -> u8 {
        todo!()
    }
}

impl fmt::Display for AppError {
    /// One line, no trailing newline. Paths are printed with `Path::display`.
    ///
    /// | Variant            | Message                            |
    /// |--------------------|------------------------------------|
    /// | `Usage(msg)`       | `msg`, unchanged                   |
    /// | `NotFound(path)`   | `file not found: {path}`           |
    /// | `InvalidUtf8(path)`| `file is not valid UTF-8: {path}`  |
    /// | `EmptyInput(path)` | `file is empty: {path}`            |
    /// | `Io(_)`            | `I/O error` (details come from `source()`) |
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Error for AppError {
    /// `Io(err)` → `Some(err)`. Every other variant → `None`.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!()
    }
}

impl From<io::Error> for AppError {
    /// Always `Io(err)`, whatever `err.kind()` is. `From` has no path to put in `NotFound`;
    /// [`read_text`] is where error kinds get special treatment.
    fn from(err: io::Error) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub path: PathBuf,
    /// Lines with more chars than this count as long. Always at least 1.
    pub max_width: usize,
}

/// Parses the full argument list as returned by `std::env::args()`.
///
/// `args[0]` is the program name (like PHP's `$argv[0]`) and is ignored, whatever it contains.
/// The remaining arguments must be exactly one `FILE` plus, optionally, `--max-width N`
/// before or after it. Without the option, `max_width` is [`DEFAULT_MAX_WIDTH`]. If the
/// option appears more than once, the last value wins.
///
/// Arguments are processed left to right, and the first problem found is returned as
/// `AppError::Usage` with exactly this message:
///
/// - `--max-width` is the last argument: `"--max-width needs a value"`.
/// - Its value doesn't parse as a `usize`, or is `0`: `"invalid --max-width value: {value}"`.
///   The argument right after `--max-width` is always taken as its value, even if it starts with `-`.
/// - Any other argument starting with `-`: `"unknown option: {arg}"`. So `--max-width=10` is unknown.
/// - A second `FILE`: `"unexpected argument: {arg}"`.
/// - No `FILE` at all, checked after every argument has been read (an empty slice
///   included): `"missing FILE argument"`.
pub fn parse_args(args: &[String]) -> Result<Config, AppError> {
    todo!()
}

/// Reads the whole file at `path` as UTF-8 text.
///
/// - The file doesn't exist (`io::ErrorKind::NotFound`) → `NotFound(path)`.
/// - Its contents aren't valid UTF-8 → `InvalidUtf8(path)`.
/// - Any other I/O failure → `Io(err)` with the original `io::Error`.
///
/// An empty file is not an error here: it returns `Ok` with an empty string.
pub fn read_text(path: &Path) -> Result<String, AppError> {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub long_lines: usize,
}

/// Counts `text`. Never fails: every `&str` is valid input, and `""` gives all zeros.
///
/// - `lines`: lines as [`str::lines`] splits them. A final line without `\n` still counts,
///   a trailing `\n` doesn't start an extra line, and `\r\n` is one line ending.
/// - `words`: words as [`str::split_whitespace`] splits them.
/// - `chars`: every `char` in `text`, line endings included. Not bytes.
/// - `long_lines`: lines (without their line ending) with more than `max_width` chars.
pub fn count(text: &str, max_width: usize) -> Counts {
    todo!()
}

/// Writes `counts` to `out` as four lines, each ending in `\n`:
///
/// ```text
/// lines: 4
/// words: 39
/// chars: 220
/// long lines: 1
/// ```
///
/// A failed write is returned as `AppError::Io`.
pub fn write_report(out: &mut impl Write, counts: &Counts) -> Result<(), AppError> {
    todo!()
}

/// Reads `config.path`, counts it with `config.max_width`, and writes the report to `out`.
///
/// Errors from [`read_text`] and [`write_report`] are returned unchanged. If the text has
/// no words (empty or whitespace only), returns `EmptyInput(config.path)` and writes nothing.
pub fn run(config: &Config, out: &mut impl Write) -> Result<(), AppError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_REPORT: &str = "lines: 4\nwords: 39\nchars: 220\nlong lines: 1\n";

    fn fixtures_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn fixture(name: &str) -> PathBuf {
        fixtures_dir().join(name)
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    fn usage_message(list: &[&str]) -> String {
        match parse_args(&args(list)) {
            Err(AppError::Usage(message)) => message,
            other => panic!("expected a Usage error, got {other:?}"),
        }
    }

    fn config(path: PathBuf) -> Config {
        Config {
            path,
            max_width: DEFAULT_MAX_WIDTH,
        }
    }

    struct ClosedPipe;

    impl Write for ClosedPipe {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "reader went away",
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn displays_one_line_messages() {
        assert_eq!(
            AppError::Usage("missing FILE argument".to_string()).to_string(),
            "missing FILE argument"
        );
        assert_eq!(
            AppError::NotFound(PathBuf::from("notes.txt")).to_string(),
            "file not found: notes.txt"
        );
        assert_eq!(
            AppError::InvalidUtf8(PathBuf::from("data/latin1.txt")).to_string(),
            "file is not valid UTF-8: data/latin1.txt"
        );
        assert_eq!(
            AppError::EmptyInput(PathBuf::from("empty.txt")).to_string(),
            "file is empty: empty.txt"
        );
        assert_eq!(
            AppError::Io(io::Error::other("disk full")).to_string(),
            "I/O error"
        );
    }

    #[test]
    fn only_io_errors_have_a_source() {
        let err = AppError::Io(io::Error::other("disk full"));
        let source = err.source().expect("Io should expose the io::Error");
        assert_eq!(source.to_string(), "disk full");

        assert!(AppError::Usage("bad".to_string()).source().is_none());
        assert!(AppError::NotFound(PathBuf::from("a")).source().is_none());
        assert!(AppError::InvalidUtf8(PathBuf::from("a")).source().is_none());
        assert!(AppError::EmptyInput(PathBuf::from("a")).source().is_none());
    }

    #[test]
    fn from_wraps_every_io_error_in_io() {
        let err = AppError::from(io::Error::new(io::ErrorKind::NotFound, "gone"));
        assert!(matches!(&err, AppError::Io(inner) if inner.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn question_mark_uses_from() {
        fn read_missing() -> Result<String, AppError> {
            let text = std::fs::read_to_string(fixture("does-not-exist.txt"))?;
            Ok(text)
        }

        assert!(matches!(read_missing(), Err(AppError::Io(_))));
    }

    #[test]
    fn usage_errors_exit_with_2_and_others_with_1() {
        assert_eq!(AppError::Usage("bad".to_string()).exit_code(), 2);
        assert_eq!(AppError::NotFound(PathBuf::from("a")).exit_code(), 1);
        assert_eq!(AppError::InvalidUtf8(PathBuf::from("a")).exit_code(), 1);
        assert_eq!(AppError::EmptyInput(PathBuf::from("a")).exit_code(), 1);
        assert_eq!(AppError::Io(io::Error::other("x")).exit_code(), 1);
    }

    #[test]
    fn parses_a_file_with_the_default_width() {
        let config = parse_args(&args(&["errors-io", "notes.txt"])).unwrap();
        assert_eq!(
            config,
            Config {
                path: PathBuf::from("notes.txt"),
                max_width: DEFAULT_MAX_WIDTH,
            }
        );
    }

    #[test]
    fn parses_max_width_before_or_after_the_file() {
        let expected = Config {
            path: PathBuf::from("notes.txt"),
            max_width: 100,
        };
        let before = parse_args(&args(&["errors-io", "--max-width", "100", "notes.txt"]));
        let after = parse_args(&args(&["errors-io", "notes.txt", "--max-width", "100"]));
        assert_eq!(before.unwrap(), expected);
        assert_eq!(after.unwrap(), expected);
    }

    #[test]
    fn last_max_width_wins() {
        let list = ["errors-io", "--max-width", "5", "a.txt", "--max-width", "7"];
        assert_eq!(parse_args(&args(&list)).unwrap().max_width, 7);
    }

    #[test]
    fn ignores_the_program_name() {
        let config = parse_args(&args(&["--max-width", "notes.txt"])).unwrap();
        assert_eq!(config.path, PathBuf::from("notes.txt"));
    }

    #[test]
    fn reports_a_missing_file_argument() {
        assert_eq!(usage_message(&[]), "missing FILE argument");
        assert_eq!(usage_message(&["errors-io"]), "missing FILE argument");
        assert_eq!(
            usage_message(&["errors-io", "--max-width", "5"]),
            "missing FILE argument"
        );
    }

    #[test]
    fn rejects_a_second_file() {
        assert_eq!(
            usage_message(&["errors-io", "a.txt", "b.txt"]),
            "unexpected argument: b.txt"
        );
    }

    #[test]
    fn rejects_unknown_options() {
        assert_eq!(
            usage_message(&["errors-io", "-v", "a.txt"]),
            "unknown option: -v"
        );
        assert_eq!(
            usage_message(&["errors-io", "a.txt", "--max-width=10"]),
            "unknown option: --max-width=10"
        );
    }

    #[test]
    fn rejects_a_missing_max_width_value() {
        assert_eq!(
            usage_message(&["errors-io", "a.txt", "--max-width"]),
            "--max-width needs a value"
        );
    }

    #[test]
    fn rejects_invalid_max_width_values() {
        for value in ["abc", "0", "-5", "1.5", ""] {
            assert_eq!(
                usage_message(&["errors-io", "--max-width", value, "a.txt"]),
                format!("invalid --max-width value: {value}")
            );
        }
    }

    #[test]
    fn reports_the_first_problem_from_the_left() {
        assert_eq!(
            usage_message(&["errors-io", "a.txt", "b.txt", "--bogus"]),
            "unexpected argument: b.txt"
        );
        assert_eq!(
            usage_message(&["errors-io", "--bogus", "a.txt", "b.txt"]),
            "unknown option: --bogus"
        );
        assert_eq!(
            usage_message(&["errors-io", "--max-width", "0", "--bogus"]),
            "invalid --max-width value: 0"
        );
    }

    #[test]
    fn reads_a_utf8_file() {
        let text = read_text(&fixture("sample.txt")).unwrap();
        assert_eq!(text, include_str!("../tests/fixtures/sample.txt"));
    }

    #[test]
    fn reads_an_empty_file_as_empty_text() {
        assert_eq!(read_text(&fixture("empty.txt")).unwrap(), "");
    }

    #[test]
    fn maps_a_missing_file_to_not_found() {
        let path = fixture("does-not-exist.txt");
        match read_text(&path) {
            Err(AppError::NotFound(reported)) => assert_eq!(reported, path),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn maps_invalid_utf8_to_invalid_utf8() {
        let path = fixture("latin1.txt");
        match read_text(&path) {
            Err(AppError::InvalidUtf8(reported)) => assert_eq!(reported, path),
            other => panic!("expected InvalidUtf8, got {other:?}"),
        }
    }

    #[test]
    fn maps_other_failures_to_io() {
        // Reading a directory fails with neither NotFound nor InvalidData.
        let result = read_text(&fixtures_dir());
        assert!(matches!(result, Err(AppError::Io(_))), "got {result:?}");
    }

    #[test]
    fn counts_lines_words_and_chars() {
        let expected = Counts {
            lines: 2,
            words: 3,
            chars: 14,
            long_lines: 0,
        };
        assert_eq!(count("one two\nthree\n", 80), expected);
        assert_eq!(count("", 80), Counts::default());
    }

    #[test]
    fn counts_lines_like_str_lines() {
        assert_eq!(count("a", 80).lines, 1);
        assert_eq!(count("a\n", 80).lines, 1);
        assert_eq!(count("a\nb", 80).lines, 2);
        assert_eq!(count("a\n\nb\n", 80).lines, 3);
        assert_eq!(count("\n", 80).lines, 1);
        assert_eq!(count("a\r\nb\r\n", 80).lines, 2);
    }

    #[test]
    fn splits_words_on_any_whitespace() {
        assert_eq!(count("  one\ttwo  \n three ", 80).words, 3);
        assert_eq!(count(" \t\n ", 80).words, 0);
    }

    #[test]
    fn counts_chars_not_bytes() {
        let text = "café 🦀\n";
        assert_eq!(text.len(), 11);
        assert_eq!(count(text, 80).chars, 7);
        assert_eq!(count("a\r\nb\r\n", 80).chars, 6);
    }

    #[test]
    fn long_lines_have_more_than_max_width_chars() {
        assert_eq!(count("12345\n123456\n", 5).long_lines, 1);
        assert_eq!(count("ééééé\n", 5).long_lines, 0);
        assert_eq!(count("abc\r\n", 3).long_lines, 0);
        assert_eq!(count("a\n\nbb\nccc", 1).long_lines, 2);
    }

    #[test]
    fn writes_the_report() {
        let counts = Counts {
            lines: 4,
            words: 39,
            chars: 220,
            long_lines: 1,
        };
        let mut out = Vec::new();
        write_report(&mut out, &counts).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), SAMPLE_REPORT);
    }

    #[test]
    fn write_failures_become_io_errors() {
        let result = write_report(&mut ClosedPipe, &Counts::default());
        assert!(
            matches!(&result, Err(AppError::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe),
            "got {result:?}"
        );
    }

    #[test]
    fn run_writes_the_report_for_a_file() {
        let mut out = Vec::new();
        run(&config(fixture("sample.txt")), &mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), SAMPLE_REPORT);
    }

    #[test]
    fn run_uses_the_configured_max_width() {
        let config = Config {
            path: fixture("sample.txt"),
            max_width: 40,
        };
        let mut out = Vec::new();
        run(&config, &mut out).unwrap();
        assert!(String::from_utf8(out).unwrap().ends_with("long lines: 3\n"));
    }

    #[test]
    fn run_rejects_files_without_words() {
        for name in ["empty.txt", "blank.txt"] {
            let path = fixture(name);
            let mut out = Vec::new();
            match run(&config(path.clone()), &mut out) {
                Err(AppError::EmptyInput(reported)) => assert_eq!(reported, path),
                other => panic!("expected EmptyInput for {name}, got {other:?}"),
            }
            assert!(out.is_empty(), "nothing should be written for {name}");
        }
    }

    #[test]
    fn run_passes_errors_through() {
        let mut out = Vec::new();
        let missing = run(&config(fixture("does-not-exist.txt")), &mut out);
        assert!(
            matches!(missing, Err(AppError::NotFound(_))),
            "got {missing:?}"
        );

        let latin1 = run(&config(fixture("latin1.txt")), &mut out);
        assert!(
            matches!(latin1, Err(AppError::InvalidUtf8(_))),
            "got {latin1:?}"
        );

        let closed = run(&config(fixture("sample.txt")), &mut ClosedPipe);
        assert!(matches!(closed, Err(AppError::Io(_))), "got {closed:?}");
    }
}
