# Lesson 08 — Modules and tests

Build a Markdown table-of-contents CLI as a library plus a thin binary: implement every `todo!()` in `src/` and `tests/` until the tests pass.

## Run

Inside the container:

```bash
cd /workspace/lessons/08-modules-testing
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- README.md
```

Until a function is implemented, its parameters show `unused variable` warnings, and `MAX_LEVEL`, `is_fence`, and `taken` show dead-code warnings. Don't run `cargo fix`; it renames parameters to `_name`.

`cargo test` stops after the first test target that fails (the unit tests). Add `--no-fail-fast` to see the integration and doc tests too.

## Notes

### Modules are declared, not discovered

```text
src/lib.rs              crate root: `pub mod markdown;`
src/markdown.rs         module `markdown`: `mod fence;` and `pub mod slug;`
src/markdown/fence.rs   module `markdown::fence`
src/markdown/slug.rs    module `markdown::slug`
```

- In `lib.rs`, `mod markdown;` loads `src/markdown.rs`. In `markdown.rs`, `mod fence;` loads `src/markdown/fence.rs`: a module's children live in a folder named after it. The older `src/markdown/mod.rs` layout also works; having both for one module is an error.
- A `.rs` file that no `mod` declares is not compiled at all. No error, no warning.
- PHP: Composer's PSR-4 autoloader finds `App\Markdown\Slug` in `src/Markdown/Slug.php` on first use. Rust builds the whole module tree at compile time from `mod` declarations. The namespace still mirrors the folders, but `mod` creates the link; `use` never loads anything.
- `mod name { … }` defines a module inline. `#[cfg(test)] mod tests { … }` is one.

### Paths and `use`

```rust
crate::markdown::slug::Slugger   // absolute, from this crate's root   ≈ \App\Markdown\Slug\Slugger
super::Heading                   // from the parent module
self::fence::is_fence            // from the current module
modules_testing::generate_toc    // from another crate: how main.rs and tests/ see the library

use crate::markdown::slug::Slugger;   // a shortcut in this module's scope
use crate::markdown::{self, Heading}; // `markdown` itself and `markdown::Heading`
pub use markdown::Heading;            // re-export: `modules_testing::Heading` now works too
```

- `use` works like PHP's file-level `use`, but per **module**: a `use` in `lib.rs` doesn't reach `toc.rs`. `use super::*;` in `mod tests` imports everything from the parent.
- `pub use` builds a facade. Callers write `modules_testing::Heading`; the type stays in `markdown`. Both paths name the same type (`tests/api.rs` checks it).
- The package is `modules-testing`; in code the crate is `modules_testing`. Hyphens become underscores.

### Visibility is per module, not per class

| PHP | Rust | Visible to |
|---|---|---|
| `public` | `pub` | anyone who can reach the path |
| `private` | (nothing) | the current module and its children |
| `protected` | — | no inheritance, so no equivalent |
| `@internal` (checked only by static analysers) | `pub(crate)` | the whole crate, enforced by the compiler |
| — | `pub(super)` | the parent module and its children |

- PHP's `private` stops at the class; Rust's stops at the module. All code in `slug.rs` can read `Slugger.taken`; `toc.rs` can't.
- Struct fields are private even on a `pub struct`. Mark fields `pub` (`Heading`, `Options`) or provide a constructor (`Slugger::new`): outside its module, `Slugger { taken: … }` doesn't compile. Enum variants are always as public as the enum.
- A path is only as visible as its least visible step. `mod fence;` is private, so `is_fence` would be unreachable outside `markdown` even if it were `pub`.
- Children see their ancestors' private items; parents don't see their children's. That's how `mod tests` reaches private functions.
- `dead_code` warns about items that nothing uses and nothing outside the crate can reach. Items reachable from the crate root through `pub` paths are exempt, because another crate might use them. A `pub fn` inside a private module still warns.

### One package, two crates

| | Library | Binary |
|---|---|---|
| Root file | `src/lib.rs` | `src/main.rs` |
| Name in paths | `modules_testing` | — |
| Sees | its own items, following visibility | only the library's `pub` items |

- `main.rs` is compiled as a separate crate that depends on the library, so `pub(crate)` items are invisible there.
- Gotcha: don't write `mod cli;` in `main.rs`. It compiles `cli.rs` a second time inside the binary crate, where `crate::markdown` doesn't exist. Use `modules_testing::cli`.
- Keep `main.rs` thin: arguments, files, stdout/stderr, exit code. Logic in the library is testable without starting a process. PHP: a console command that only delegates to a service class.
- `fn main() -> ExitCode` hands a status code to the shell. `std::process::exit(code)` also works, but it ends the process immediately without running destructors (an unflushed `BufWriter` loses data).
- More binaries go in `src/bin/<name>.rs`; each is its own crate using the same library.

### Three kinds of tests

| Kind | Where | Compiled as | Can call | Run only these |
|---|---|---|---|---|
| Unit | `#[cfg(test)] mod tests` in each module | part of the library | private items too | `cargo test --lib` |
| Integration | `tests/*.rs` | one crate per file | `pub` items only | `cargo test --test cli` |
| Doc | code blocks in `///` and `//!` comments | code outside the library | `pub` items only | `cargo test --doc` |

- `#[cfg(test)]` compiles the module only into the library's own unit-test binary. `cargo test` also builds the library without `cfg(test)` for `tests/`, doc tests and `main.rs`, so a `#[cfg(test)]` helper in `src/` is invisible to integration tests. PHPUnit keeps every test in `tests/`; Rust keeps unit tests next to the code, which is why they can reach private items.
- Shared integration helpers go in `tests/common/mod.rs`, pulled in with `mod common;`. A `tests/common.rs` would be compiled as a test crate of its own.
- `env!("CARGO_BIN_EXE_modules-testing")` is the path of the compiled binary. Cargo builds it before the integration tests, so `tests/cli.rs` can run it with `std::process::Command` — like Symfony's `Process` in a PHPUnit test. `env!("CARGO_MANIFEST_DIR")` is the crate directory, used to locate `tests/fixtures/`.
- A test can return `Result<(), E>` and use `?`; an `Err` fails it (`tests/api.rs`).
- `#[should_panic(expected = "…")]` expects a panic. `#[ignore]` skips a test unless you pass `-- --ignored`.

### Doc tests

```rust
/// One-line summary.
///
/// # Examples
///
/// ```
/// use modules_testing::markdown::slug::slugify;
///
/// assert_eq!(slugify("Run the tests"), "run-the-tests");
/// ```
pub fn slugify(title: &str) -> String { … }
```

- Every code block in a doc comment is compiled and run as a test from outside the crate, so it imports public paths. Examples can't silently go stale. They run for library crates only, not for `main.rs`.
- Lines starting with `# ` are compiled but hidden from the rendered docs.
- Block annotations: `text` (not Rust), `no_run` (compile only), `ignore`, `should_panic`, `compile_fail`.
- `compile_fail` passes if compilation fails **for any reason**, typos included. `src/markdown.rs` has two that check `fence` and `MAX_LEVEL` stay hidden. Make `MAX_LEVEL` `pub` and one of them fails.
- Edition 2024 merges doc tests into one binary where it can; `compile_fail` blocks still build alone. That's why doc tests print two result lines.

PHP: phpDocumentor renders docblocks the same way, but nothing runs their examples.

### Running a subset

| Command | Runs |
|---|---|
| `cargo test slug` | unit and integration tests whose name contains `slug`, such as `markdown::slug::tests::…` |
| `cargo test --lib` | unit tests only |
| `cargo test --test cli` | `tests/cli.rs` only |
| `cargo test --doc` | doc tests only; `cargo test --doc slugify` filters them |
| `cargo test --no-fail-fast` | every target, even after one fails |
| `cargo test -- --no-capture` | also prints the `println!` output of passing tests |

Arguments after `--` go to the test harness, not to Cargo. With this image's Cargo, a name filter without `--doc` skips doc tests.

### `cargo doc`

- `///` documents the next item; `//!` documents the enclosing module or crate (top of `lib.rs`, `markdown.rs`).
- Contents are Markdown. Conventional sections: `# Examples`, `# Errors`, `# Panics`.
- Intra-doc links such as ``[`Slugger`]`` or ``[`crate::markdown::slug::Slugger`]`` become hyperlinks; broken ones are warnings.
- `cargo doc --no-deps` documents this crate but not its dependencies. Add `--document-private-items` to include private items.
- `cargo doc --open` can't open a browser from inside Docker. Write the HTML into the lesson folder (`target/` is gitignored) from inside the container:

```bash
cargo doc --no-deps --target-dir target
```

Then open it from the repository root on your Mac:

```bash
open lessons/08-modules-testing/target/doc/modules_testing/index.html
```

## Exercise

Implement in this order. `cargo test --lib` gives the fastest feedback until step 7.

1. `src/markdown/fence.rs`: `is_fence`. Then write the test `rejects_lines_that_are_not_fences`. `fence` is private, but its own unit tests can call it.
2. `src/markdown.rs`: `parse_heading`, then `headings`.
   - `trim_start_matches('#')` plus a length comparison counts the `#`s.
   - `usize::from(MAX_LEVEL)` and `u8::try_from(count)` convert between integer types.
   - `headings` calls the child's `pub(super)` function as `fence::is_fence(line)` and keeps an `in_fence` flag.
3. `src/markdown/slug.rs`: `slugify` (`to_lowercase`, then `chars()`, a `match` per char, `collect()`), then `Slugger::new` and `Slugger::unique` (loop while `self.taken.contains(…)`).
4. `src/toc.rs`: `render`. Add `use crate::markdown::slug::Slugger;` first.
   - Compute every heading's anchor before filtering.
   - The shallowest listed level is `Iterator::min` over the listed levels; it returns an `Option`.
   - `"  ".repeat(n)` builds the indent.
5. `src/lib.rs`: `generate_toc` — one line through `markdown::headings` and `toc::render`.
6. `src/cli.rs`: `parse_args`. Import the crate-private default with `use crate::markdown::MAX_LEVEL;`. Iterate with `let mut args = args.iter(); while let Some(arg) = args.next() { … }` so `--max-level` can take its value with `args.next()`.
7. `src/main.rs`: follow the steps in its `//!` comment. Import with `use modules_testing::…`, not `mod`. `eprintln!` writes to stderr; `ExitCode::from(2)` and `ExitCode::SUCCESS` are exit codes.
8. `tests/cli.rs`: write `missing_file_exits_with_code_1` using the `run` helper.
9. Generate the docs (command above) and read your crate's pages, then try `--document-private-items`.

Then run it on real files: `cargo run -- --max-level 2 ../../ROADMAP.md`.

Optional extension: move `Slugger` into `src/markdown/slug/slugger.rs` and re-export it from `slug.rs` with `pub use`. Every test, doc test, and `toc.rs` must compile unchanged. Which `use` does `slugger.rs` need to reach `slugify`?

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass, including the two tests you wrote.
- You can explain:
  - why `parse_args` lives in the library while `std::env::args` stays in `main.rs`
  - why `fence`'s unit tests can call `is_fence` but `tests/cli.rs` and `main.rs` can't
  - what `pub(crate)` on `MAX_LEVEL` allows and forbids, and what the `compile_fail` doc tests do and don't prove
  - why `main.rs` imports `modules_testing::cli` instead of declaring `mod cli;`
  - why the shared helpers live in `tests/common/mod.rs`, not `tests/common.rs`
