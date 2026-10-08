//! Runs the compiled `errors-io` binary and checks stdout, stderr, and the exit code.

use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn errors_io(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_errors-io"))
        .args(args)
        .output()
        .expect("failed to start the errors-io binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn prints_the_report_and_exits_with_0() {
    let output = errors_io(&[&fixture("sample.txt")]);
    assert_eq!(stderr(&output), "");
    assert_eq!(
        stdout(&output),
        "lines: 4\nwords: 39\nchars: 220\nlong lines: 1\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn passes_max_width_to_the_count() {
    let output = errors_io(&[&fixture("sample.txt"), "--max-width", "40"]);
    assert_eq!(
        stdout(&output),
        "lines: 4\nwords: 39\nchars: 220\nlong lines: 3\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn usage_errors_print_the_usage_line_and_exit_with_2() {
    let output = errors_io(&[]);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "error: missing FILE argument\nusage: errors-io [--max-width N] FILE\n"
    );
    assert_eq!(output.status.code(), Some(2));

    let output = errors_io(&["--max-width", "zero", "a.txt"]);
    assert_eq!(
        stderr(&output),
        "error: invalid --max-width value: zero\nusage: errors-io [--max-width N] FILE\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn input_errors_exit_with_1() {
    let output = errors_io(&["does-not-exist.txt"]);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "error: file not found: does-not-exist.txt\n"
    );
    assert_eq!(output.status.code(), Some(1));

    let path = fixture("latin1.txt");
    let output = errors_io(&[&path]);
    assert_eq!(
        stderr(&output),
        format!("error: file is not valid UTF-8: {path}\n")
    );
    assert_eq!(output.status.code(), Some(1));

    let path = fixture("blank.txt");
    let output = errors_io(&[&path]);
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), format!("error: file is empty: {path}\n"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn io_errors_print_their_cause() {
    // A directory can't be read as a file; the OS message after "caused by: " varies by platform.
    let output = errors_io(&[&fixture("")]);
    let stderr = stderr(&output);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 2, "stderr was {stderr:?}");
    assert_eq!(lines[0], "error: I/O error");
    assert!(
        lines[1].starts_with("  caused by: "),
        "stderr was {stderr:?}"
    );
    assert!(
        lines[1].len() > "  caused by: ".len(),
        "stderr was {stderr:?}"
    );
    assert_eq!(output.status.code(), Some(1));
}
