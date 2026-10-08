# Lesson 15 — Native CLI and filesystem work

Build `native-cli`, a directory size scanner: implement every `todo!()` in `src/lib.rs` and `src/main.rs` until the tests pass.

## Run

Inside the container:

```bash
cd /workspace/lessons/15-native-cli
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- /workspace/lessons --top 5 --binary
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

Docker runs as root, and root ignores permission bits, so `an_unreadable_directory_is_recorded_and_the_scan_continues` returns early there. To exercise it, run the test binary as user `nobody` through a Cargo runner:

```bash
cargo test \
  --config 'target."cfg(unix)".runner = "setpriv --reuid=65534 --regid=65534 --clear-groups"' --lib unreadable
```

## Notes

### clap derive vs symfony/console

```rust
use clap::Parser;

/// Resize images.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Input file
    input: PathBuf,                          // no #[arg]: positional, required
    #[arg(short, long, default_value_t = 800)]
    width: u32,                              // -w, --width <WIDTH>, parsed as u32
    #[arg(long)]
    dry_run: bool,                           // flag: --dry-run
}

let args = Args::parse();                    // invalid input: prints the error, exits with 2
```

| symfony/console | clap |
|---|---|
| `addArgument('input', InputArgument::REQUIRED)` | a plain field; optional if it's an `Option` or has a default |
| `addOption('width', 'w', InputOption::VALUE_REQUIRED, '', 800)` | `#[arg(short, long, default_value_t = 800)] width: u32` |
| `InputOption::VALUE_NONE` | a `bool` field |
| `$input->getOption('width')` is `mixed`; you cast and validate | a typed field: `--width abc` is rejected before your code runs |
| `Command::SUCCESS` / `FAILURE` / `INVALID` (0/1/2) | `ExitCode::SUCCESS` / `ExitCode::FAILURE` / `ExitCode::from(2)`; clap's usage errors also exit 2 |

- The struct's doc comment becomes `about` in `--help`; a field's doc comment becomes its help text. A `//` comment at the end of a `///` line becomes part of that text.
- `default_value_t = 10` takes a typed value. `default_value = "."` takes a string clap parses, which suits `PathBuf`.
- `short` alone uses the field's first letter. `--top` needs `-n`, so write `short = 'n'`.
- `--help` and `--version` are generated. `Args::parse()` calls `std::process::exit` on errors; `Args::try_parse()` returns them instead.

### `Path` and `PathBuf`

`PathBuf` is to `Path` what `String` is to `&str` (lesson 04): accept `&Path`, store `PathBuf`. Unlike `str`, a path isn't guaranteed to be UTF-8 — a Linux file name can be any bytes except `/` and NUL — so `Path` wraps an `OsStr`.

```rust
let p = Path::new("src").join("main.rs");   // "src/main.rs"; no string concatenation
p.file_name();                               // Some("main.rs"), an &OsStr
p.extension();                               // Some("rs")
p.parent();                                  // Some("src")
p.display();                                 // for printing; invalid UTF-8 becomes U+FFFD
p.to_str();                                  // Option<&str>: None if not UTF-8
p.to_string_lossy();                         // Cow<str>, always succeeds
```

- PHP's `SplFileInfo::getExtension()` returns `""` when there is no extension. `Path::extension()` returns `None`, and `Some("")` for `"file."`. For `.gitignore`, PHP returns `"gitignore"`, while Rust returns `None` because a leading dot doesn't start an extension.
- `PathBuf` compares by components: `a/b` sorts before `a.txt`, though `"a/b" > "a.txt"` as strings.

### Directory traversal with `read_dir`

```rust
for entry in fs::read_dir(dir)? {            // Err: the directory can't be opened
    let entry = entry?;                      // Err: a failure while reading the listing
    let path = entry.path();                 // `dir` joined with the entry's name
    let file_type = fs::symlink_metadata(&path)?.file_type();
    // exactly one of: is_file(), is_dir(), is_symlink(), or none (socket, FIFO, device)
}
```

- Two layers of `Result`: opening the directory, and each entry.
- `.` and `..` are never yielded, like PHP's `FilesystemIterator::SKIP_DOTS`.
- **Order is unspecified**: whatever the filesystem returns, and it differs between ext4 and APFS. Sort when output must be deterministic. Use `BTreeMap` for sorted keys; `HashMap` iteration order is random, unlike PHP arrays.
- Recursion is the obvious shape. A `Vec` of directories still to visit, used as a stack, avoids deep call stacks. Either passes the tests.

| Follows symlinks | Doesn't follow | PHP |
|---|---|---|
| `fs::metadata`, `Path::is_dir`, `Path::is_file`, `Path::exists` | `fs::symlink_metadata`, `DirEntry::file_type`, `DirEntry::metadata` | `stat`/`is_dir` vs `lstat`/`is_link` |

Choosing from the left column makes `sub/parent -> ..` a trap: the scan re-enters the same tree one level deeper each time, until the OS gives up with `ELOOP` ("Too many levels of symbolic links", ~40 links on Linux). With a second loop such as `self -> root`, the work doubles at every level, so it effectively never finishes. PHP's `RecursiveDirectoryIterator` only descends into symlinked directories with `FOLLOW_SYMLINKS`, and it has the same problem when you set the flag. This lesson's policy matches `find` and `du`: report symlinks, never follow them. The root is the exception. `fs::metadata` follows it, so a symlinked root such as macOS's `/tmp` works, like `find -H`.

Sizes are `Metadata::len`, the apparent size. `du` reports allocated blocks and counts hard links once, so totals can differ.

### Error policy: fatal vs per-entry

`?` aborts the whole function. That's right for the root: without it there's nothing to report. It's wrong for one unreadable directory among thousands, so record the failure and move on:

```rust
let entries = match fs::read_dir(&path) {
    Ok(entries) => entries,
    Err(err) => {
        report.errors.push(EntryError { path, kind: err.kind(), message: err.to_string() });
        continue;                            // `continue` has type `!`, so this arm fits any type
    }
};
```

- PHP's `RecursiveIteratorIterator` throws `UnexpectedValueException` on an unreadable directory, or skips it silently with `CATCH_GET_CHILD`. Here the failure becomes data: the report lists it, and the exit code (1) says the result is partial, as with `du` and `find`.
- `io::Error` is neither `Clone` nor `PartialEq`. `EntryError` stores `kind()` and `to_string()` so `ScanReport` stays comparable in tests. `ScanError` keeps the `io::Error` and exposes it through `Error::source`.

### Buffered I/O

```rust
let mut out = BufWriter::new(io::stdout().lock());
write_report(&report, &mut out)?;
out.flush()?;
```

- `Stdout` is line-buffered: each `println!` takes a lock and issues a `write` syscall per line. `BufWriter` collects output into 8 KiB writes, similar to PHP's `ob_start()`.
- `BufWriter` flushes on drop, but drop ignores errors, and `std::process::exit` skips destructors entirely. Flush explicitly, then return an `ExitCode` from `main`.
- `println!` panics if stdout fails, for example when `native-cli | head -1` closes the pipe early. Rust ignores `SIGPIPE`, so the failure arrives as `ErrorKind::BrokenPipe`, and `writeln!` returns it for you to handle.
- `write_report` takes `&mut impl Write`, so tests can pass a `Vec<u8>` and `main` can pass the `BufWriter`.

`BufReader` adds `BufRead` to any reader. Its default buffer is 8 KiB, the same size as `SNIFF_LEN`.

```rust
let buf = reader.fill_buf()?;   // borrows the internal buffer; an empty slice means end of input
let n = buf.len();              // take what you need from `buf`…
reader.consume(n);              // …before this call, which needs `reader` mutably again
```

`fill_buf` returns whatever is buffered, which can be less than you need. Loop until you have enough or get an empty slice. `reader.take(SNIFF_LEN as u64).read_to_end(&mut bytes)` is the simpler alternative that copies the bytes. To call these on a parameter declared `reader: R`, write `mut reader: R`. `mut` belongs to the binding, not to the function's signature.

### Binary data

- `String` is always UTF-8 and `Vec<u8>` holds arbitrary bytes. PHP has one byte-string type for both. `&[u8]` is a byte slice, and `b"PK\x03\x04"` is a byte-string literal.
- The heuristic is that a NUL byte in the first 8 KiB means binary. Git does the same with 8000 bytes. UTF-16 text contains NULs, so it counts as binary, which is acceptable for a size report.
- `bytes.contains(&0)` searches a slice.

### Release builds and native executables

| | `cargo build` | `cargo build --release` |
|---|---|---|
| Profile | `dev`: no optimization, debug info | `release`: `opt-level = 3`, debug info stripped |
| Output | `<target-dir>/debug/native-cli` | `<target-dir>/release/native-cli` |
| This binary (Docker, Linux) | about 15 MB | about 1.2 MB |

In Docker, `CARGO_TARGET_DIR=/cargo-target`, so binaries are written to the `cargo-target` volume, not under the lesson directory:

```bash
cargo build --release
ls -lh /cargo-target/release/native-cli
/cargo-target/release/native-cli /workspace --top 3
```

That one file is the whole program. Another machine needs no `php` binary, extensions, or `vendor/`, only the same OS, the same CPU architecture, and a compatible libc. Docker builds a **Linux ELF** binary for the container's architecture (`aarch64-unknown-linux-gnu` on Apple Silicon; `rustc -vV` shows it as `host:`). macOS refuses to run it with `exec format error`. For a macOS (Mach-O) binary, install rustup on the Mac and run the same `cargo build --release` natively.

```bash
rustup target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
ldd /cargo-target/aarch64-unknown-linux-musl/release/native-cli
```

`--target` writes to `<target-dir>/<triple>/release/`. `ldd` prints `not a dynamic executable`: the musl binary is static. On an Intel Mac the container is x86_64, so use `x86_64-unknown-linux-musl` instead.

- A target triple is `arch-vendor-os[-env]`. `-gnu` binaries link glibc dynamically, so they need a compatible glibc where they run. `-musl` binaries are statically linked.
- `rustup target add` installs only the target's standard library. Linking for another CPU or OS also needs a linker for that target: building `x86_64-unknown-linux-musl` in the arm64 container fails with a `cc` error. `cross` (prepared build containers) or `cargo-zigbuild` (Zig as the linker) solve this for Linux targets. Apple targets need the macOS SDK, so build those on a Mac.
- A target added by `rustup` lives in the container, not the image, so it's gone after `docker compose down`. Add it again in the new container.

Size and speed settings in `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"      # optimize for size ("s" is similar); the default 3 optimizes for speed
lto = true           # optimize across crates; slower builds
codegen-units = 1    # better optimization; slower builds
panic = "abort"      # no unwinding: a panic ends the process immediately
strip = true         # remove symbols too; release already strips debug info by default
```

Together these settings shrink this binary from about 1.2 MB to about 0.5 MB. `opt-level = "z"` can make code slower, so measure before keeping it.

Target-specific code is selected at compile time, for the **target** rather than the machine running the compiler:

```rust
#[cfg(unix)]                                 // the item exists only on Unix-like targets
use std::os::unix::fs::PermissionsExt;

if cfg!(target_os = "macos") { /* … */ }     // both branches must compile; one is dead code
```

The symlink, socket, and permission tests are `#[cfg(unix)]`, because `std::os::unix` doesn't exist on Windows.

## Exercise

Implement in this order:

1. `format_size`: return early below 1024, then divide an `f64` by 1024 in a loop. For the `1024.0` rule, check what `{:.1}` would print, not just `value >= 1024.0`.
2. `extension_key`: `Path::extension` gives `Option<&OsStr>`. Use `to_string_lossy`, `to_lowercase`, and `unwrap_or_default`.
3. `looks_binary`: a `fill_buf`/`consume` loop with a `remaining` counter. Never examine more than `remaining` bytes of a buffer.
4. `top_files`: take the `Vec` by value, then `sort_by` with `Ordering::then_with`, then `truncate`. Nothing needs cloning.
5. `ScanError`: `Display` with `write!(f, …)`; `source` returns `Some(source)` for `Root`.
6. `scan`, in passes (its tests follow `// --- scan` in `src/lib.rs`):
   - root: `fs::metadata` and `fs::read_dir`, with `map_err` into `ScanError::Root`;
   - one level: `symlink_metadata` per entry, the four kinds, and `by_extension.entry(key).or_default()`;
   - recursion or an explicit stack, the error policy (`match` with `continue` instead of `?`), and binary sniffing with `File::open` and `BufReader::new`;
   - finally, sort `symlinks` and `errors` and call `top_files`.
7. `write_report`: one `writeln!(out, …)?` per line. Width and alignment come from the format spec (`{:>10}`, `{:<10}`).
8. `main.rs`: the doc comment on `main` is the contract. Write the `Args` struct, call `Args::parse()`, `scan`, write through `BufWriter`, `flush`, and return an `ExitCode`. `tests/cli.rs` runs the real binary through `env!("CARGO_BIN_EXE_native-cli")`.
9. Build the release binary, scan `/workspace` with it (command above), and compare its size with `/cargo-target/debug/native-cli`.

Optional extension:

- Count each hard-linked file once, as `du` does: `std::os::unix::fs::MetadataExt::{dev, ino}` behind `#[cfg(unix)]`.
- Recognize executables by magic bytes: ELF `7F 45 4C 46`, 64-bit Mach-O `CF FA ED FE`, PE `4D 5A`. Scan `/cargo-target/release` to see what Docker produced.
- Add `--max-depth <N>`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo build --release` succeeds, and the release binary scans `/workspace`.
- The unreadable-directory test passes when run as `nobody` (the `setpriv` runner command under Run). Without it, `cargo test` skips the error-policy test.
- You can explain:
  - what `Path::is_dir` would do with `sub/parent -> ..`, and how `symlink_metadata` prevents it
  - why a missing root is an `Err` while an unreadable subdirectory is data in the report
  - why `main` flushes the `BufWriter` explicitly instead of relying on drop
  - why the Docker-built binary doesn't run on macOS, and what `--target` and a linker each contribute
