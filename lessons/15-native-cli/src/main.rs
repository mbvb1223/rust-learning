use std::process::ExitCode;

/// `native-cli [OPTIONS] [PATH]` — define the arguments with `clap`'s derive API:
///
/// - `[PATH]`: positional field `path` (clap derives `[PATH]` from the field name), the
///   directory to scan, default `.`;
/// - `-n, --top <N>`: how many of the largest files to list, default 10;
/// - `-b, --binary`: count binary files (sets `ScanOptions::detect_binary`).
///
/// Writes `native_cli::write_report` output to stdout through a `BufWriter<StdoutLock>`,
/// flushed explicitly. Exit codes:
///
/// - 0: report written, no entry errors;
/// - 1: report written, but `report.errors` is not empty;
/// - 2: invalid arguments (clap prints the message and exits), a `ScanError`, or a failure
///   writing stdout. For the last two, print `error: {e}` to stderr; a `ScanError` writes
///   nothing to stdout.
fn main() -> ExitCode {
    todo!()
}
