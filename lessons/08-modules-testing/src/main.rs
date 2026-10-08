//! Prints the table of contents of a Markdown file.
//!
//! A separate crate from the library: it sees only `pub` items, through paths that start with
//! `modules_testing::`. All the I/O lives here, and nothing else:
//!
//! 1. Collect `std::env::args()` without the program name; parse them with `cli::parse_args`.
//!    On error, print two lines to stderr, `error: {err}` then `cli::USAGE`, and exit with 2.
//! 2. Read the file with `std::fs::read_to_string`. On error, print
//!    `error: cannot read {path}: {io_error}` to stderr and exit with 1.
//! 3. Print `generate_toc(...)` to stdout exactly as returned (`print!`, not `println!`), and
//!    exit with 0. A document without headings prints nothing.

use std::process::ExitCode;

fn main() -> ExitCode {
    todo!()
}
