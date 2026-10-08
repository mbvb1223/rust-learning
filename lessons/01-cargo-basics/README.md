# Lesson 01 — Cargo and basic syntax

Build a temperature converter: implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

From the repository root:

```bash
docker compose run --rm -w /workspace/lessons/01-cargo-basics rust cargo test
docker compose run --rm -w /workspace/lessons/01-cargo-basics rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/01-cargo-basics rust cargo clippy --all-targets -- -D warnings
docker compose run --rm -w /workspace/lessons/01-cargo-basics rust cargo run
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`; it renames them to `_name`.

## Notes

### Cargo

| Command | What it does | PHP equivalent |
|---|---|---|
| `cargo new name` | Create a project | `composer init` |
| `cargo check` | Type-check without building — fastest feedback | `php -l` + PHPStan |
| `cargo build` | Compile to `target/debug/` (`--release` for optimized) | — |
| `cargo run` | Build and run `src/main.rs` | `php script.php` |
| `cargo test` | Build and run all `#[test]` functions | `vendor/bin/phpunit` |

`Cargo.toml` ≈ `composer.json`. `Cargo.lock` ≈ `composer.lock` — commit it for applications.

This crate has a library (`src/lib.rs`, the exercises) and a binary (`src/main.rs`, which uses the library). Lesson 08 explains the split.

### Variables and types

```rust
let count = 5;            // i32 inferred, immutable
let mut total = 0.0;      // f64 inferred, mutable
total += 1.5;

let input = "  42 ";
let input = input.trim(); // shadowing: a new variable with the same name
```

- Variables are immutable unless declared `mut`.
- Shadowing creates a new variable, which can have a different type. `x = …` mutates; `let x = …` shadows.
- Default integer type is `i32`, default float is `f64`. Others: `i8`–`i128`, `u8`–`u128`, `isize`/`usize`, `f32`, `bool`, `char`.
- **No implicit conversion.** `5 + 1.5` does not compile. Convert explicitly: `f64::from(5)` (lossless) or `5 as f64` (may lose data).
- **Integer division truncates:** `9 / 5 == 1`. In PHP `9 / 5` is `1.8`; Rust behaves like `intdiv(9, 5)`.
- `&'static str` is the type of a string literal such as `"hot"`. Lesson 04 covers strings.

### Expressions

A block evaluates to its last expression when it has **no semicolon**. Functions return that value; use `return` only for early exits.

```rust
fn sign(n: i32) -> &'static str {
    if n < 0 { "negative" } else { "non-negative" } // `if` is an expression: no ternary needed
}
```

### Loops

```rust
for n in 1..=3 {}          // 1, 2, 3   (`1..3` excludes 3)
while condition {}
let found = loop {
    break 42;              // only `loop` can return a value
};
```

`for` and `while` always evaluate to `()`.

### Tests

```rust
#[test]
fn adds() {
    assert_eq!(1 + 1, 2);
    assert!(2 > 1, "optional message");
}
```

Floats are inexact. The tests compare them with a tolerance (`assert_close`) instead of `assert_eq!`. The `#[cfg(test)] mod tests` wrapper is explained in lesson 08.

## Exercise

Implement in this order:

1. `celsius_to_fahrenheit`, `fahrenheit_to_celsius`. Use float literals (`9.0`, not `9`).
2. `celsius_to_fahrenheit_whole`: integers only. Operation order matters: `37 * 9 / 5` ≠ `37 * (9 / 5)`.
3. `round_to`: use shadowing for the intermediate value. `f64::round` rounds half away from zero, like PHP's `round()`.
4. `describe`: a single `if`/`else if` expression, no `return`.
5. `average_fahrenheit`: a `for` loop over an inclusive range, plus `let mut` accumulators.
6. `first_celsius_reaching`: `loop` with `break value`.

Then run `cargo run` to print the conversion table.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `37 * (9 / 5)` gives the wrong answer
  - why `describe` needs no `return`
  - the difference between `let x = …` and `x = …` on an existing variable
