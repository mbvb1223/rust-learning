//! Command-line arguments → [`Options`]. No I/O here, so it's unit-testable; `main.rs` reads
//! the real arguments and the file.

use std::fmt;

pub const USAGE: &str = "Usage: modules-testing [--max-level N] <FILE>";

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub path: String,
    pub max_level: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    MissingPath,
    /// `--max-level` was the last argument.
    MissingLevel,
    /// The value after `--max-level` isn't an integer from 1 to 6.
    InvalidLevel(String),
    /// An unknown flag, or a second path.
    UnexpectedArgument(String),
}

// `Display` is lesson 09. It's what `format!("{err}")` uses.
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingPath => write!(f, "missing <FILE> argument"),
            CliError::MissingLevel => write!(f, "--max-level needs a value"),
            CliError::InvalidLevel(value) => {
                write!(f, "invalid --max-level '{value}': expected 1 to 6")
            }
            CliError::UnexpectedArgument(arg) => write!(f, "unexpected argument '{arg}'"),
        }
    }
}

/// Parses the arguments that follow the program name: `[--max-level N] <FILE>`.
///
/// - The flag may come before or after the path. Repeated, the last one wins.
/// - `max_level` defaults to 6 (the crate's `MAX_LEVEL`).
/// - The argument after `--max-level` is always its value, even if it starts with `-`.
///   It must parse as an integer from 1 to 6, else [`CliError::InvalidLevel`] with that value.
///   No argument after it → [`CliError::MissingLevel`].
/// - Any other argument starting with `-` → [`CliError::UnexpectedArgument`] with that argument.
/// - The first remaining argument is the path; a second one → [`CliError::UnexpectedArgument`]
///   with that argument.
/// - No path at all → [`CliError::MissingPath`].
///
/// Arguments are checked left to right and the first error is returned.
///
/// # Errors
///
/// Every [`CliError`] variant, as described above.
///
/// # Examples
///
/// ```
/// use modules_testing::cli::{Options, parse_args};
///
/// let args = vec!["--max-level".to_string(), "2".to_string(), "README.md".to_string()];
/// assert_eq!(
///     parse_args(&args),
///     Ok(Options { path: "README.md".to_string(), max_level: 2 }),
/// );
/// ```
pub fn parse_args(args: &[String]) -> Result<Options, CliError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    fn options(path: &str, max_level: u8) -> Options {
        Options {
            path: path.to_string(),
            max_level,
        }
    }

    #[test]
    fn path_alone_uses_the_default_level() {
        assert_eq!(parse_args(&args(&["notes.md"])), Ok(options("notes.md", 6)));
    }

    #[test]
    fn flag_goes_before_or_after_the_path() {
        let expected = Ok(options("notes.md", 2));
        assert_eq!(
            parse_args(&args(&["--max-level", "2", "notes.md"])),
            expected
        );
        assert_eq!(
            parse_args(&args(&["notes.md", "--max-level", "2"])),
            expected
        );
        assert_eq!(
            parse_args(&args(&["--max-level", "5", "notes.md", "--max-level", "2"])),
            expected
        );
    }

    #[test]
    fn rejects_levels_outside_one_to_six() {
        for value in ["0", "7", "two", "-1", ""] {
            assert_eq!(
                parse_args(&args(&["a.md", "--max-level", value])),
                Err(CliError::InvalidLevel(value.to_string())),
                "{value:?}"
            );
        }
    }

    #[test]
    fn reports_missing_values_and_paths() {
        assert_eq!(parse_args(&args(&[])), Err(CliError::MissingPath));
        assert_eq!(
            parse_args(&args(&["--max-level", "3"])),
            Err(CliError::MissingPath)
        );
        assert_eq!(
            parse_args(&args(&["a.md", "--max-level"])),
            Err(CliError::MissingLevel)
        );
    }

    #[test]
    fn rejects_unknown_flags_and_extra_paths() {
        assert_eq!(
            parse_args(&args(&["--help", "a.md"])),
            Err(CliError::UnexpectedArgument("--help".to_string()))
        );
        assert_eq!(
            parse_args(&args(&["a.md", "b.md"])),
            Err(CliError::UnexpectedArgument("b.md".to_string()))
        );
    }

    #[test]
    fn returns_the_first_error_from_the_left() {
        assert_eq!(
            parse_args(&args(&["--max-level", "9", "--verbose"])),
            Err(CliError::InvalidLevel("9".to_string()))
        );
        assert_eq!(
            parse_args(&args(&["a.md", "b.md", "--max-level", "9"])),
            Err(CliError::UnexpectedArgument("b.md".to_string()))
        );
    }
}
