//! The `errors-io` binary: a thin shell around the library. `tests/cli.rs` checks its behaviour.

use std::process::ExitCode;

/// 1. Collect `std::env::args()` into a `Vec<String>` and pass it to `parse_args`.
/// 2. `run` the resulting config with standard output as the writer.
/// 3. On success, print nothing else and return `ExitCode::SUCCESS`.
/// 4. On any error from step 1 or 2, write these lines to standard error:
///    - `error: {err}`
///    - one `  caused by: {cause}` line (two-space indent) for each error in the
///      `source()` chain, starting with `err.source()`
///    - for `Usage` errors only, finally the `USAGE` line
///
///    then return `ExitCode::from(err.exit_code())`.
fn main() -> ExitCode {
    todo!()
}
