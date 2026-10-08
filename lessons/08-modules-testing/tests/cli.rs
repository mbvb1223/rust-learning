//! CLI tests: run the real binary as a child process, like a shell would.

mod common;

use std::process::{Command, Output};

use modules_testing::cli::{CliError, USAGE};

// Cargo builds the binary before integration tests and passes its path in this variable.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_modules-testing"))
        .args(args)
        .output()
        .expect("failed to start the binary")
}

fn sample_path() -> String {
    common::fixture("sample.md").to_string_lossy().into_owned()
}

#[test]
fn prints_the_toc_of_a_file() {
    let output = run(&[&sample_path()]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), common::SAMPLE_TOC);
    assert!(output.stderr.is_empty());
}

#[test]
fn passes_max_level_to_the_library() {
    let output = run(&["--max-level", "2", &sample_path()]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        common::SAMPLE_TOC_MAX_2
    );
}

#[test]
fn usage_errors_exit_with_code_2() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("error: {}\n{USAGE}\n", CliError::MissingPath)
    );

    let output = run(&["--max-level", "9", &sample_path()]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!(
            "error: {}\n{USAGE}\n",
            CliError::InvalidLevel("9".to_string())
        )
    );
}

#[test]
fn missing_file_exits_with_code_1() {
    todo!("run the binary on a path that doesn't exist; check the exit code, stdout, and stderr")
}
