# Lesson 15 — Native CLI and filesystem work

Build `native-cli`, a directory size scanner: implement every `todo!()` in `src/lib.rs` and `src/main.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/15-native-cli
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- /workspace/lessons --top 5 --binary
```

Don't run `cargo fix`.

Docker runs as root, so the unreadable-directory test returns early. Run it as user `nobody`:

```bash
cargo test \
  --config 'target."cfg(unix)".runner = "setpriv --reuid=65534 --regid=65534 --clear-groups"' --lib unreadable
```

## Notes

### Arguments with clap

Describe the arguments as a struct, and `#[derive(Parser)]` writes the parser and `--help`:

```rust
use clap::Parser;

/// Resize images
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file
    input: PathBuf,
    /// Target width
    #[arg(short, long, default_value_t = 800)]
    width: u32,
    #[arg(long)]
    dry_run: bool,
}

let args = Args::parse();
println!("{args:?}");
```

```text
$ resize photo.jpg -w 640 --dry-run
Args { input: "photo.jpg", width: 640, dry_run: true }
$ resize photo.jpg --width abc
error: invalid value 'abc' for '--width <WIDTH>': invalid digit found in string
```

- A field without `#[arg]` is positional. A `bool` is a flag. `///` comments become the `--help` text.
- `short` uses the field's first letter. `--top` needs `short = 'n'`.
- `default_value_t = 800` takes a typed value. `default_value = "."` takes a string, which suits `PathBuf`.
- Bad input exits with code 2. `main` can return `ExitCode::SUCCESS` (0), `ExitCode::FAILURE` (1), or `ExitCode::from(2)`.

Like symfony/console, but typed: `--width abc` never reaches your code.

### Path and PathBuf

`PathBuf` is to `Path` what `String` is to `&str`: accept `&Path`, store `PathBuf`.

```rust
use std::path::Path;

let p = Path::new("src").join("main.rs");
println!("{}", p.display());                           // src/main.rs
println!("{:?}", p.file_name());                       // Some("main.rs")
println!("{:?}", p.parent());                          // Some("src")
println!("{:?}", p.extension());                       // Some("rs")
println!("{:?}", Path::new("a.tar.gz").extension());   // Some("gz")
println!("{:?}", Path::new("Makefile").extension());   // None
println!("{:?}", Path::new(".gitignore").extension()); // None
println!("{:?}", Path::new("file.").extension());      // Some("")
```

A path isn't always UTF-8, so `extension` returns an `&OsStr`. `to_string_lossy()` turns it into text.

PHP's `pathinfo('.gitignore', PATHINFO_EXTENSION)` returns `"gitignore"`. Rust returns `None`: a leading dot doesn't start an extension.

### Reading a directory

`fs::read_dir` lists one directory. Opening it can fail, and so can each entry:

```rust
use std::fs;

for entry in fs::read_dir("src")? {
    let entry = entry?;
    println!("{}", entry.path().display());
}
// src/main.rs
// src/lib.rs
```

- `.` and `..` are skipped.
- The order is whatever the filesystem returns. Sort when output must be stable. A `BTreeMap` keeps its keys sorted.
- To go deeper, recurse, or keep a `Vec` of directories still to visit.

### Symlinks: don't follow them

`fs::symlink_metadata` describes the entry itself. `fs::metadata` follows a symlink to its target, so a link like `sub/parent -> ..` would send the scan around in circles.

```rust
let meta = fs::symlink_metadata(&path)?;
let file_type = meta.file_type();
if file_type.is_symlink() {
    // report it, never follow it
} else if file_type.is_dir() {
    // scan it
} else if file_type.is_file() {
    let size = meta.len(); // bytes
} else {
    // socket, FIFO, device: "other"
}
```

- Follow symlinks: `fs::metadata`, `Path::is_dir`, `Path::is_file`, `Path::exists`.
- Don't follow: `fs::symlink_metadata`, `DirEntry::file_type`, `DirEntry::metadata`.
- The root is the exception: check it with `fs::metadata`, so a symlinked root still works.

### Fatal vs per-entry errors

`?` stops the whole function. That's right for the root. For one unreadable folder among thousands, record the error and `continue`:

```rust
let entries = match fs::read_dir(&path) {
    Ok(entries) => entries,
    Err(err) => {
        report.errors.push(EntryError {
            path,
            kind: err.kind(),
            message: err.to_string(),
        });
        continue;
    }
};
```

- `io::Error` is neither `Clone` nor `PartialEq`, so `EntryError` stores `kind()` and `to_string()`.
- `ScanError` keeps the `io::Error` and returns it from `Error::source`.

### Buffered output

`BufWriter` collects small writes into big ones, like PHP's `ob_start()`. Flush it yourself at the end:

```rust
use std::io::{self, BufWriter, Write};

let mut out = BufWriter::new(io::stdout().lock());
writeln!(out, "Files: {}", 3)?; // Files: 3
out.flush()?;
```

- Dropping a `BufWriter` also flushes, but ignores errors.
- `write_report` takes `&mut impl Write`, so tests pass a `Vec<u8>` and `main` passes the `BufWriter`.

### Formatting numbers

```rust
println!("{:.1}", 1536.0 / 1024.0);      // 1.5
println!("{:.1}", 1_048_575.0 / 1024.0); // 1024.0
println!("[{:>6}]", "abc");              // [   abc]
println!("[{:<6}]", "abc");              // [abc   ]
```

`{:.1}` rounds to one decimal, so a value below 1024 can still print as `1024.0`. `>` right-aligns and `<` left-aligns within the width.

### Reading bytes with BufRead

`fill_buf` returns the bytes buffered so far. `consume` marks them as read:

```rust
use std::io::{BufRead, BufReader};

let mut reader = BufReader::with_capacity(4, &b"hello"[..]);
let buf = reader.fill_buf()?;
println!("{buf:?}"); // [104, 101, 108, 108]
let n = buf.len();
reader.consume(n);
println!("{:?}", reader.fill_buf()?); // [111]
```

- `fill_buf` may return fewer bytes than you need. Loop until you have enough, or it returns an empty slice (end of input).
- Use `buf` before `consume`: `buf` borrows `reader`.
- To call these on a parameter, declare it `mut reader: R`.

### Binary data

`String` is always UTF-8. `&[u8]` holds any bytes. This lesson calls a file binary if its first 8 KiB contain a NUL byte, as Git does:

```rust
let bytes: &[u8] = b"PK\x03\x04\0"; // a byte-string literal
println!("{}", bytes.contains(&0));   // true
```

### Release builds

`cargo build --release` builds an optimized binary: about 1.2 MB here, against about 15 MB for a debug build.

```bash
cargo build --release
ls -lh /cargo-target/release/native-cli
/cargo-target/release/native-cli /workspace --top 3
```

That one file is the whole program. The machine that runs it needs no `php`, extensions, or `vendor/`.

### Building for other targets

Docker builds a Linux binary for the container's CPU, so macOS refuses to run it (`exec format error`). For a Mac binary, install rustup on the Mac and build there.

```bash
rustup target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
ldd /cargo-target/aarch64-unknown-linux-musl/release/native-cli   # not a dynamic executable
```

- A target triple is `arch-vendor-os[-env]`. `-musl` builds a static binary. On an Intel Mac, use `x86_64-unknown-linux-musl`.
- `rustup target add` adds only the standard library. Building for another CPU or OS also needs a linker for it.

## Exercise

Implement in this order:

1. `format_size`: return early below 1024, then divide an `f64` by 1024 in a loop. For the `1024.0` rule, check what `{:.1}` would print.
2. `extension_key`: `Path::extension` gives `Option<&OsStr>`; use `to_string_lossy`, `to_lowercase`, and `unwrap_or_default`.
3. `looks_binary`: a `fill_buf`/`consume` loop with a `remaining` counter; never look past `remaining` bytes.
4. `top_files`: take the `Vec` by value, `sort_by` with `Ordering::then_with`, then `truncate`.
5. `ScanError`: `Display` with `write!(f, …)`; `source` returns `Some(source)` for `Root`.
6. `scan`, in passes (its tests follow `// --- scan` in `src/lib.rs`):
   - root: `fs::metadata` and `fs::read_dir`, with `map_err` into `ScanError::Root`;
   - one level: `symlink_metadata` per entry, the four kinds, `by_extension.entry(key).or_default()`;
   - recursion or a stack, the error policy (`match` + `continue`), and sniffing with `File::open` + `BufReader::new`;
   - finally, sort `symlinks` and `errors`, and call `top_files`.
7. `write_report`: one `writeln!(out, …)?` per line; alignment from `{:>10}` and `{:<10}`.
8. `main.rs`: the doc comment on `main` is the contract. `Args` struct, `Args::parse()`, `scan`, `BufWriter`, `flush`, return an `ExitCode`.
9. Build the release binary, scan `/workspace` with it (see "Release builds"), and compare its size with `/cargo-target/debug/native-cli`.

Optional: count hard-linked files once, like `du` (`std::os::unix::fs::MetadataExt::{dev, ino}` behind `#[cfg(unix)]`).

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo build --release` succeeds, and the release binary scans `/workspace`.
- The unreadable-directory test passes when run as `nobody` (the `setpriv` command under Run).
- You can explain:
  - what `Path::is_dir` would do with `sub/parent -> ..`, and how `symlink_metadata` prevents it
  - why a missing root is an `Err` while an unreadable subdirectory is data in the report
  - why the Docker-built binary doesn't run on macOS
