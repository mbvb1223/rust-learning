# Lesson 05 — Errors and file I/O

Build a `wc`-style CLI that counts a text file and reports bad input as typed errors: implement every `todo!()` in `src/lib.rs` and `src/main.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/05-errors-io
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -q -- tests/fixtures/sample.txt
cargo run -q -- --max-width 40 tests/fixtures/sample.txt
cargo run -q -- missing.txt; echo "exit code: $?"
```

Arguments after `--` go to your program, not to Cargo; `-q` hides Cargo's own output. Don't run `cargo fix`.

## Notes

### Result

A function that can fail returns `Result<T, E>`: either `Ok(value)` or `Err(error)`. There are no exceptions; you handle the error with `match`.

```rust
match "42".parse::<u32>() {
    Ok(n) => println!("got {n}"), // got 42
    Err(err) => println!("not a number: {err}"),
}
```

In PHP, `@throws` is only a comment. In Rust the error is part of the return type, so a caller can't forget it.

### The `?` operator

`?` unwraps an `Ok`, or returns the `Err` from the current function right away:

```rust
use std::num::ParseIntError;

fn double(s: &str) -> Result<u32, ParseIntError> {
    let n = s.parse::<u32>()?;
    Ok(n * 2)
}

println!("{:?}", double("21"));  // Ok(42)
println!("{:?}", double("abc")); // Err(ParseIntError { kind: InvalidDigit })
```

`?` only works inside a function that returns `Result` (or `Option`). It also converts the error with `From::from` (next sections).

### Converting errors

- `result.map_err(|err| …)` changes the error and keeps the value.
- `option.ok_or_else(|| …)` turns `None` into an `Err`, so you can use `?` on it.
- `result.and_then(|value| …)` chains another step that can fail.

```rust
let first: Option<&str> = None;
let value = first.ok_or_else(|| "missing value".to_string());
println!("{value:?}"); // Err("missing value")
```

Use `map_err` when the new error needs context `From` can't know, such as the file path.

### A custom error type

An error type is usually an enum with one variant per kind of failure:

```rust
use std::{fmt, io};

#[derive(Debug)]
pub enum ConfigError {
    MissingKey(String),
    Io(io::Error),
}
```

`Display` gives the message, like PHP's `getMessage()`:

```rust
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingKey(key) => write!(f, "missing key: {key}"),
            ConfigError::Io(_) => write!(f, "cannot read config"),
        }
    }
}
```

`Error::source()` gives the cause, like `getPrevious()`:

```rust
impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::MissingKey(_) => None,
        }
    }
}
```

Show the inner error in `Display` **or** through `source()`, never both, or it gets printed twice.

### From and `?`

`impl From<io::Error>` tells `?` how to turn an `io::Error` into your type:

```rust
impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

fn load(path: &Path) -> Result<String, ConfigError> {
    let text = fs::read_to_string(path)?; // io::Error becomes ConfigError::Io
    Ok(text)
}
```

### Testing errors with `matches!`

`io::Error` can't be compared with `==`, so tests check the variant with `matches!`:

```rust
let result: Result<(), ConfigError> = Err(ConfigError::MissingKey("port".to_string()));
println!("{}", matches!(result, Err(ConfigError::MissingKey(_)))); // true
```

### Panic or Result?

- Bad input or a missing file: return `Result` and let the caller decide.
- A bug that can't happen: panic with `expect("why it can't fail")`, `assert!`, or `unreachable!()`.
- `unwrap()` is fine in tests and prototypes.

A panic ends the program with exit code 101, like an uncaught PHP `Error`. `todo!()` is a panic too.

### Reading files

`fs::read_to_string(path)` reads a whole file into a `String`. `err.kind()` tells you what went wrong:

```rust
let message = match fs::read_to_string("missing.txt") {
    Ok(text) => text,
    Err(err) if err.kind() == io::ErrorKind::NotFound => "no such file".to_string(),
    Err(err) => format!("other error: {err}"),
};
println!("{message}"); // no such file
```

- A `String` is always valid UTF-8, so a file with invalid UTF-8 fails with `io::ErrorKind::InvalidData`. `tests/fixtures/latin1.txt` is such a file: its `é` is the single byte `0xE9`.
- `io::Error` messages don't include the path. Add it yourself.

### Paths

`&Path` and `PathBuf` are the path versions of `&str` and `String`. Print a path with `.display()`:

```rust
use std::path::{Path, PathBuf};

let path = Path::new("notes.txt");
let owned: PathBuf = path.to_path_buf();
println!("{}", owned.display()); // notes.txt
```

A `&PathBuf` works wherever a `&Path` is expected.

### Writing with `io::Write`

`writeln!(out, …)` is `println!` aimed at any writer. A failed write is an `Err`, not a panic:

```rust
use std::io::{self, Write};

fn greet(out: &mut impl Write, name: &str) -> io::Result<()> {
    writeln!(out, "hello, {name}")?;
    Ok(())
}

let mut out = Vec::new();
greet(&mut out, "ann").unwrap();
println!("{:?}", String::from_utf8(out).unwrap()); // "hello, ann\n"
```

A `Vec<u8>` is an in-memory writer, like `php://memory`. Tests pass one; `main` passes `&mut io::stdout()`.

### Command-line arguments

`std::env::args()` yields the arguments as `String`s. The first one is the program name, like `$argv[0]`:

```rust
let args: Vec<String> = std::env::args().collect();
```

`while let` loops as long as a pattern matches. Unlike `for`, the body can call `next()` itself to grab an option's value:

```rust
let args = ["--width", "40", "notes.txt"];
let mut rest = args.iter();
while let Some(arg) = rest.next() {
    if *arg == "--width" {
        println!("width: {:?}", rest.next()); // width: Some("40")
    } else {
        println!("file: {arg}"); // file: notes.txt
    }
}
```

### Exit codes and stderr

`main` can return an `ExitCode`. `eprintln!` prints to stderr, like `fwrite(STDERR, …)`:

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("error: something failed");
    ExitCode::from(2)
}
```

```text
$ cargo run -q; echo $?
error: something failed
2
```

Convention: 2 for usage errors, 1 for other failures. Prefer returning `ExitCode` to `std::process::exit`, which skips cleanup.

### Integration tests

`tests/cli.rs` runs the compiled binary and checks its stdout, stderr, and exit code. Lesson 08 covers the `tests/` folder.

## Exercise

The doc comments in `src/` are the full spec.

Part A — `AppError` in `src/lib.rs`:

1. `Display`: one `match` with `write!(f, …)`; `path.display()` for paths.
2. `Error::source`: only `Io` has a source.
3. `From<io::Error>`: always `Io`, whatever the error's kind.
4. `exit_code`: `2` for `Usage`, `1` for everything else.

Part B — the pipeline:

5. `parse_args`: `let mut rest = args.iter().skip(1);` with `while let`; `rest.next().ok_or_else(…)?` for a missing value; `Ok(n) if n > 0` in a `match` on `parse::<usize>()`; track the file as `Option<PathBuf>`.
6. `read_text`: `fs::read_to_string` plus `map_err` with a `match` on `err.kind()`; `path.to_path_buf()` for the error.
7. `count`: `lines()`, `split_whitespace()`, `chars().count()`.
8. `write_report`: `writeln!(out, …)?` four times. The `?` works because of step 3.
9. `run`: `?` on each step and one `if` for `EmptyInput`.

Part C — `src/main.rs`:

10. `main`: `parse_args(&args).and_then(|config| run(&config, &mut io::stdout()))`, then `match`. Walk the causes with `while let Some(cause) = source { …; source = cause.source(); }` (needs `use std::error::Error;`).

Optional: `cargo add thiserror`, replace your three impls with `#[derive(thiserror::Error)]`, `#[error("…")]`, and `#[from]` (paths: `#[error("file not found: {}", .0.display())]`), and keep the tests green.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - what `?` does, and why `write_report` can use plain `?` while `read_text` needs `map_err`
  - why `AppError::Io` shows its details through `source()` and not in `Display`
  - when to panic and when to return `Result`
