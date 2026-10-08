//! Runs the compiled `native-cli` binary like a user would.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use native_cli::{ScanOptions, scan, write_report};

fn native_cli() -> Command {
    // Cargo builds the binary before integration tests and exposes its path.
    Command::new(env!("CARGO_BIN_EXE_native-cli"))
}

fn run(cmd: &mut Command) -> (Output, String, String) {
    let output = cmd.output().expect("run native-cli");
    let stdout = String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout");
    let stderr = String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr");
    (output, stdout, stderr)
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("a.txt"), "hello\n").unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/data.bin"), [0u8; 2048]).unwrap();
    fs::write(root.join("sub/notes.md"), "# notes\n".repeat(20)).unwrap();
    dir
}

fn expected_report(root: &Path, opts: ScanOptions) -> String {
    let report = scan(root, &opts).unwrap();
    let mut out = Vec::new();
    write_report(&report, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn help_lists_the_options() {
    let (output, stdout, _) = run(native_cli().arg("--help"));
    assert!(output.status.success());
    for option in ["--top", "--binary", "[PATH]"] {
        assert!(stdout.contains(option), "missing {option} in:\n{stdout}");
    }
}

#[test]
fn prints_the_report_and_exits_0() {
    let dir = fixture();
    let (output, stdout, stderr) = run(native_cli()
        .arg(dir.path())
        .args(["--top", "2", "--binary"]));

    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
    let opts = ScanOptions {
        top: 2,
        detect_binary: true,
    };
    assert_eq!(stdout, expected_report(dir.path(), opts));
    assert!(stdout.contains("Files: 3 (1 binary)\n"));
}

#[test]
fn short_flags_work() {
    let dir = fixture();
    let (output, stdout, _) = run(native_cli().args(["-n", "1", "-b"]).arg(dir.path()));

    assert_eq!(output.status.code(), Some(0));
    let opts = ScanOptions {
        top: 1,
        detect_binary: true,
    };
    assert_eq!(stdout, expected_report(dir.path(), opts));
}

#[test]
fn defaults_to_the_current_directory_and_top_10() {
    let dir = fixture();
    for i in 0..9 {
        fs::write(dir.path().join(format!("f{i}.log")), "x").unwrap();
    }
    let (output, stdout, _) = run(native_cli().current_dir(dir.path()));

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.starts_with("Root: .\n"), "{stdout}");
    assert!(
        stdout.contains("Files: 12\n"),
        "binary detection is off by default"
    );
    let largest = concat!(
        "Largest files:\n",
        "     2.0 KiB  ./sub/data.bin\n",
        "       160 B  ./sub/notes.md\n",
        "         6 B  ./a.txt\n",
    );
    assert!(stdout.contains(largest), "{stdout}");
    let listed = stdout
        .split("Largest files:\n")
        .nth(1)
        .unwrap()
        .lines()
        .take_while(|line| !line.is_empty())
        .count();
    assert_eq!(listed, 10, "{stdout}");
}

#[test]
fn a_missing_root_exits_2_with_an_error_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    let (output, stdout, stderr) = run(native_cli().arg(&missing));

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout.is_empty(), "stdout: {stdout}");
    let prefix = format!("error: cannot read {}: ", missing.display());
    assert!(stderr.starts_with(&prefix), "stderr: {stderr}");
}

#[test]
fn invalid_arguments_exit_2() {
    let (output, _, stderr) = run(native_cli().args(["--top", "many"]));
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("--top"), "stderr: {stderr}");
}
