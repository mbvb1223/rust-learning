# Lesson 05 — Errors and file I/O

Build a `wc`-style CLI that counts a text file and reports missing files, invalid UTF-8, empty input, and bad arguments: implement every `todo!()` in `src/lib.rs` and `src/main.rs` until the tests pass.

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

Arguments after `--` go to your program, not to Cargo. `-q` hides Cargo's own output.

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

## Notes

### `Result` instead of exceptions

```rust
enum Result<T, E> { Ok(T), Err(E) }     // in std, always in scope

match "42".parse::<u32>() {
    Ok(n) => println!("got {n}"),
    Err(err) => eprintln!("not a number: {err}"),
}
```

- A PHP exception is invisible in the signature; `@throws` is only a comment. In Rust the failure is part of the return type: `fn read_text(path: &Path) -> Result<String, AppError>`. Callers can't reach the `String` without handling the `Err`.
- Ignoring a returned `Result` is a compiler warning (`#[must_use]`).
- Errors are plain values. Nothing unwinds the stack, and there is no class hierarchy to catch by parent type. "Catching by type" means matching enum variants.

| PHP | Rust |
|---|---|
| `throw new NotFound($path)` | `return Err(AppError::NotFound(path))` |
| `try { … } catch (NotFound $e) { … }` | `match result { Err(AppError::NotFound(p)) => …, … }` |
| let it bubble up | `?` |
| `@file_get_contents($f)` and `=== false` | `fs::read_to_string(f)` returns a `Result`, so the check can't be forgotten |
| `set_error_handler` turning warnings into `ErrorException` | not needed: std has no warning channel. Recoverable failures come back as `Err`; a few APIs panic instead (see "Panic or `Result`?") |
| `$e->getMessage()` / `$e->getPrevious()` | `Display` / `Error::source()` |
| uncaught `Error`, fatal error | panic |

### `?`

```rust
fn load(path: &Path) -> Result<String, AppError> {
    let text = fs::read_to_string(path)?;   // io::Error → AppError through From
    Ok(text)
}
```

`expr?` is roughly:

```rust
match expr {
    Ok(value) => value,
    Err(err) => return Err(From::from(err)),
}
```

- `?` only works inside a function returning `Result`, or `Option` (where `None` returns early).
- `From::from` is the conversion hook: `?` compiles once `impl From<io::Error> for AppError` exists. The PHP equivalent is `catch (IOException $e) { throw new AppException(previous: $e); }`, written once instead of at every call site.
- `From` only sees the error itself. When you need context (which path? which flag?), convert explicitly with `map_err`.
- `?` doesn't mix `Option` and `Result`. Convert first: `opt.ok_or_else(|| AppError::Usage("…".to_string()))?`.

| Method | Does |
|---|---|
| `result.map_err(f)` | changes the error, keeps the value |
| `opt.ok_or_else(f)` | `None` → `Err(f())` |
| `result.and_then(f)` | chains another step that can fail |
| `result.unwrap_or(default)` | discards the error and uses a default |

### A custom error type

```rust
#[derive(Debug)]
pub enum ConfigError {
    MissingKey(String),
    Io(io::Error),
}

impl fmt::Display for ConfigError {                       // ≈ getMessage()
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingKey(key) => write!(f, "missing key: {key}"),
            ConfigError::Io(_) => write!(f, "cannot read config"),
        }
    }
}

impl std::error::Error for ConfigError {                  // ≈ getPrevious()
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::MissingKey(_) => None,
        }
    }
}

impl From<io::Error> for ConfigError {                    // lets `?` convert
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}
```

- `impl Display for ConfigError { … }` implements a trait for a type, like `implements Stringable` on a PHP class. Lesson 09 covers traits.
- `std::error::Error` requires `Debug + Display`. `source()` defaults to `None`, so `impl Error for X {}` is valid.
- Show the inner error's message **either** in `Display` **or** through `source()`, not both. Error reporters walk the `source()` chain and would print it twice.
- `io::Error` is neither `Clone` nor `PartialEq`, so an enum holding it can't derive them, and `assert_eq!` on the whole `Result` won't compile. Tests use `matches!`:

```rust
assert!(matches!(result, Err(AppError::NotFound(_))));
```

### Real projects: `thiserror` and `anyhow`

You'll rarely write those three impls by hand:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing key: {0}")]
    MissingKey(String),
    #[error("cannot read config")]
    Io(#[from] io::Error),      // generates From and source()
}
```

- `thiserror`, for libraries: derives `Display`, `Error`, and `From`. Callers can still `match` on variants.
- `anyhow`, for applications: one `anyhow::Error` type for everything, plus `.context("reading config")?` to add a message. Callers can't match on variants, so use it at the top of a binary.

This lesson writes the impls by hand so you know what those macros generate.

### Panic or `Result`?

| Situation | Use |
|---|---|
| Missing file, bad argument, invalid data: caused by input or environment | `Result`; the caller decides |
| A bug: broken invariant, an index you proved in range, a "can't happen" state | panic: `expect("why it can't fail")`, `assert!`, `unreachable!()` |
| Tests and prototypes | `unwrap()` / `expect()` are fine |

- A panic unwinds the thread and, in `main`, ends the program with exit code 101. Treat it like PHP's uncaught `Error`, not as control flow. `todo!()` is a panic.
- Prefer `expect("parse_args checked this")` to `unwrap()`: the message records why failure is impossible.
- Hidden panics: `v[10]` out of range, integer overflow in debug builds, `println!` when stdout is closed (piped into `head`), and `std::env::args()` with a non-Unicode argument (`args_os()` doesn't panic). That's why `run` writes through a `Write` and returns `Result` instead of calling `println!`.

### Files: `std::fs` and `std::io`

| Rust | PHP |
|---|---|
| `fs::read_to_string(path)?` | `file_get_contents($path)`, but fails with `ErrorKind::InvalidData` on invalid UTF-8 |
| `fs::read(path)?` → `Vec<u8>` | `file_get_contents($path)` (binary-safe) |
| `String::from_utf8(bytes)` → `Result<String, FromUtf8Error>` | `mb_check_encoding($s, 'UTF-8')` |
| `fs::write(path, data)?` | `file_put_contents($path, $data)` |
| `err.kind() == io::ErrorKind::NotFound` | reading the warning text or `error_get_last()` |

```rust
match fs::read_to_string(path) {
    Ok(text) => …,
    Err(err) if err.kind() == io::ErrorKind::NotFound => …,
    Err(err) => …,
}
```

- PHP strings are bytes. A Rust `String` is always valid UTF-8, so reading a Latin-1 file into a `String` is an error, not mojibake.
- Don't call `path.exists()` before reading: the file can disappear in between. Read it, then handle `NotFound`.
- `io::Error` messages don't include the path (`No such file or directory (os error 2)`). Add it yourself; that's why `NotFound` carries a `PathBuf`.
- `&Path` / `PathBuf` are the path versions of `&str` / `String`. Paths aren't guaranteed to be UTF-8, so print them with `path.display()`. A `&PathBuf` coerces to `&Path`.

### Writing: `io::Write`

```rust
fn greet(out: &mut impl Write, name: &str) -> io::Result<()> {
    writeln!(out, "hello, {name}")?;   // like fwrite($stream, "hello, $name\n"), but returns io::Result<()>
    Ok(())
}
```

- `out: &mut impl Write` accepts any writer, much like a PHP function taking a stream resource: `io::stdout()`, a `File`, a `Vec<u8>`, a socket. Lesson 09 explains `impl Trait`.
- `writeln!` is `println!` aimed at a writer. A failed write is an `Err` you handle with `?`, not a panic.
- `Vec<u8>` is an in-memory writer, like `php://memory`. The tests pass one and compare the bytes; `main` passes `&mut io::stdout()`, the real target.
- `writeln!` calls a `Write` method, so it needs `use std::io::Write;` in scope. `src/lib.rs` already has it.

### Command-line arguments and exit codes

```rust
let args: Vec<String> = std::env::args().collect();   // args[0] is the program, like $argv[0]
eprintln!("error: {err}");                            // stderr, like fwrite(STDERR, …)

fn main() -> ExitCode {
    match work() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(2),
    }
}
```

- Returning `ExitCode` from `main` drops everything normally: files close, `BufWriter`s flush. `std::process::exit(2)` ends the process immediately without running destructors. This is unlike PHP's `exit(2)`, which still runs shutdown functions and `__destruct`.
- `fn main() -> Result<(), Box<dyn Error>>` also works. An `Err` prints its **Debug** form (`Error: NotFound("x.txt")`) and exits with 1. Fine for prototypes; this lesson wants exact messages and exit codes.
- Convention: exit code 2 for usage errors, 1 for other failures.
- Real CLIs parse arguments with `clap` (lesson 15). Parsing by hand here is practice with `Option`, `Result`, and `?`.

### Integration tests

`tests/cli.rs` runs the compiled binary with `std::process::Command` and checks stdout, stderr, and the exit code. Cargo provides the binary's path as `env!("CARGO_BIN_EXE_errors-io")`. Lesson 08 covers the `tests/` folder. In `tests/fixtures/latin1.txt` the `é` is the single byte `0xE9`: valid Latin-1, invalid UTF-8.

## Exercise

Implement in this order. The doc comments in `src/` are the full specification.

Part A — `AppError` in `src/lib.rs`:

1. `Display`: one `match` with `write!(f, …)`. Use `path.display()` for paths.
2. `Error::source`: only `Io` has a source.
3. `From<io::Error>`: always `Io`, whatever the error's kind.
4. `exit_code`.

Part B — the pipeline:

5. `parse_args`:
   - `let mut rest = args.iter().skip(1);` then `while let Some(arg) = rest.next()`. `for arg in rest` takes ownership of the iterator, so its body can't call `rest.next()` to fetch the option's value.
   - `rest.next().ok_or_else(…)?` for a missing value.
   - `value.parse::<usize>()` returns a `Result`. A `match` with a guard (`Ok(n) if n > 0 => n`) also rejects zero.
   - Track the file as `Option<PathBuf>` and turn `None` into the error after the loop.
6. `read_text`: `fs::read_to_string` plus `map_err` with a `match` on `err.kind()`. `path.to_path_buf()` makes the owned copy the error needs.
7. `count`: `lines()`, `split_whitespace()`, `chars().count()`.
8. `write_report`: `writeln!(out, …)?` four times. The `?` compiles because of step 3.
9. `run`: `?` on each step and one `if`.

Part C — `src/main.rs`:

10. `main`: `parse_args(&args).and_then(|config| run(&config, &mut io::stdout()))`, then `match`. Walk the cause chain with `while let Some(cause) = source { …; source = cause.source(); }`. Calling `.source()` needs the trait in scope: `use std::error::Error;`.

Optional extension: run `cargo add thiserror`, replace your `Display`, `Error`, and `From` impls with `#[derive(thiserror::Error)]` plus `#[error(…)]` and `#[from]` attributes, and confirm every test still passes. For paths, write `#[error("file not found: {}", .0.display())]`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - what `?` expands to, and why `write_report` can use plain `?` while `read_text` needs `map_err`
  - why `AppError::Io` exposes its details through `source()` and not in `Display`
  - when to panic and when to return `Result`
  - why `main` returns `ExitCode` instead of calling `std::process::exit`
