# Lesson 08 — Modules and tests

Build a Markdown table-of-contents CLI as a library plus a thin binary, with unit, integration, and doc tests.

## Run

Inside the container:

```bash
cd /workspace/lessons/08-modules-testing
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- README.md
```

Unused-variable and dead-code warnings (`MAX_LEVEL`, `is_fence`, `taken`) are expected until you implement the stubs. Don't run `cargo fix`.

`cargo test` stops at the first failing test target. Add `--no-fail-fast` to see them all.

## Notes

### Modules

`mod name;` tells Rust to compile a file as a module. A module's children live in a folder named after it:

```text
src/lib.rs              pub mod markdown;
src/markdown.rs         mod fence;  pub mod slug;
src/markdown/fence.rs   module markdown::fence
src/markdown/slug.rs    module markdown::slug
```

A `.rs` file that no `mod` declares isn't compiled at all. PHP's autoloader finds a class on first use; Rust builds the whole module tree from `mod` lines.

### Paths and `use`

```rust
crate::markdown::slug::Slugger // from this crate's root
super::Heading                 // from the parent module
modules_testing::generate_toc  // from outside: main.rs and tests/

use crate::markdown::slug::Slugger; // a shortcut, for this module only
pub use markdown::Heading;          // re-export: `modules_testing::Heading` works too
```

- `use` is per module: a `use` in `lib.rs` doesn't reach `toc.rs`.
- The package `modules-testing` is the crate `modules_testing` in code: hyphens become underscores.

### Visibility

Items are private to their module unless you mark them:

- `pub`: visible wherever the path can be reached
- `pub(crate)`: this crate only
- `pub(super)`: the parent module
- nothing: this module and its children

```rust
mod shop {
    pub fn open() {}
    fn restock() {}
}

shop::open();    // ok
shop::restock(); // error[E0603]: function `restock` is private
```

PHP's `private` stops at the class; Rust's stops at the module.

- Struct fields are private even on a `pub struct`. Mark them `pub`, or add a constructor like `Slugger::new`.
- Children see their parent's private items. That's how `mod tests` reaches private functions.
- A path is only as visible as its least visible step: `fence` is private, so `is_fence` can't be reached outside `markdown`.

### Library and binary

One package holds two crates: the library (`src/lib.rs`) and the binary (`src/main.rs`). The binary is a separate crate that sees only the library's `pub` items:

```rust
// src/main.rs
use modules_testing::cli; // not `mod cli;`
```

- `mod cli;` in `main.rs` would compile `cli.rs` a second time inside the binary, where `crate::markdown` doesn't exist.
- Keep `main.rs` thin: arguments, files, printing, exit code. Logic goes in the library, where it's easy to test.
- `fn main() -> ExitCode` returns the exit status: `ExitCode::SUCCESS`, `ExitCode::from(2)`.

### Unit tests

Unit tests live in a `#[cfg(test)] mod tests` next to the code, so they can call private items:

```rust
fn double(n: i32) -> i32 {
    n * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        assert_eq!(double(2), 4);
    }
}
```

```text
test tests::doubles ... ok
```

`#[cfg(test)]` compiles the module only for unit tests. PHPUnit keeps every test in `tests/`; Rust keeps unit tests beside the code.

### Integration tests

Each file in `tests/` is a separate crate that sees only `pub` items, like any other user of your library:

```rust
// tests/api.rs
use modules_testing::generate_toc;

#[test]
fn text_without_headings_gives_an_empty_toc() {
    assert_eq!(generate_toc("Just a paragraph.\n", 6), "");
}
```

- Shared helpers go in `tests/common/mod.rs`, loaded with `mod common;`. A `tests/common.rs` would become a test crate of its own.
- `env!("CARGO_BIN_EXE_modules-testing")` is the path of the built binary, so `tests/cli.rs` can run it with `std::process::Command`.
- `env!("CARGO_MANIFEST_DIR")` is the crate folder, used to find `tests/fixtures/`.
- A test can return `Result` and use `?`; an `Err` fails it.

### Doc tests

Code blocks in `///` comments are compiled and run as tests, from outside the crate:

```rust
/// ```
/// use modules_testing::markdown::slug::slugify;
///
/// assert_eq!(slugify("Run the tests"), "run-the-tests");
/// ```
pub fn slugify(title: &str) -> String { … }
```

- Lines starting with `# ` run but are hidden in the docs.
- A `compile_fail` block passes if it fails to compile for any reason, typos included. `src/markdown.rs` has two that check `fence` and `MAX_LEVEL` stay hidden.

### Running a subset

- `cargo test slug`: tests whose name contains `slug`
- `cargo test --lib`: unit tests only
- `cargo test --test cli`: `tests/cli.rs` only
- `cargo test --doc`: doc tests only
- `cargo test --no-fail-fast`: keep going after a failing target
- `cargo test -- --no-capture`: also show `println!` output from passing tests

### `cargo doc`

`///` documents the next item; `//!` documents the module or crate it's in. Doc comments are Markdown, and ``[`Slugger`]`` becomes a link.

`cargo doc --open` can't open a browser from inside Docker. Build the HTML into the lesson folder instead:

```bash
cargo doc --no-deps --target-dir target
```

Then open it from the repository root on your Mac:

```bash
open lessons/08-modules-testing/target/doc/modules_testing/index.html
```

Add `--document-private-items` to include private items.

## Exercise

Implement in this order. `cargo test --lib` gives the fastest feedback until step 7.

1. `src/markdown/fence.rs`: `is_fence`, then write the test `rejects_lines_that_are_not_fences`.
2. `src/markdown.rs`: `parse_heading` (count `#`s with `trim_start_matches('#')` and a length comparison; convert with `usize::from(MAX_LEVEL)` and `u8::try_from(count)`), then `headings` (an `in_fence` flag and `fence::is_fence(line)`).
3. `src/markdown/slug.rs`: `slugify` (`to_lowercase`, a `match` per char, `collect()`), then `Slugger::new` and `unique` (loop while `self.taken.contains(…)`).
4. `src/toc.rs`: `render`. Add `use crate::markdown::slug::Slugger;`, compute every anchor before filtering, and indent with `"  ".repeat(n)` from the shallowest listed level (`Iterator::min`).
5. `src/lib.rs`: `generate_toc`, one line through `markdown::headings` and `toc::render`.
6. `src/cli.rs`: `parse_args`. `use crate::markdown::MAX_LEVEL;`, then `let mut args = args.iter(); while let Some(arg) = args.next() { … }` so `--max-level` can take its value with `args.next()`.
7. `src/main.rs`: follow its `//!` steps with `use modules_testing::…`, `eprintln!`, `ExitCode::from(2)`, and `ExitCode::SUCCESS`.
8. `tests/cli.rs`: write `missing_file_exits_with_code_1` with the `run` helper.
9. Build the docs (command above), read your crate's pages, then try `--document-private-items`.

Then run it on a real file: `cargo run -- --max-level 2 ../../ROADMAP.md`.

Optional: move `Slugger` into `src/markdown/slug/slugger.rs` and re-export it from `slug.rs` with `pub use`; everything else must compile unchanged.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass, including the two tests you wrote.
- You can explain:
  - why `parse_args` lives in the library while `std::env::args` stays in `main.rs`
  - why `fence`'s unit tests can call `is_fence` but `tests/cli.rs` and `main.rs` can't
  - why `main.rs` imports `modules_testing::cli` instead of declaring `mod cli;`
