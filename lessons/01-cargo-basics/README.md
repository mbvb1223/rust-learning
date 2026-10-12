# Lesson 01 — Cargo and basic syntax

Build a temperature converter in `src/lib.rs`.

## Run

There's no Rust on the Mac; everything runs in the `rust` container. From the repository root:

```bash
docker compose up -d
docker compose exec rust bash
```

Inside the container:

```bash
cd /workspace/lessons/01-cargo-basics
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run
```

Unfinished functions show `unused variable` warnings. Don't run `cargo fix`.

## Notes

### Cargo

Cargo builds, runs, and tests your project. `Cargo.toml` is like `composer.json`, and `Cargo.lock` like `composer.lock`.

- `cargo new name`: create a project
- `cargo check`: type-check only, the fastest feedback
- `cargo build`: compile to `target/debug/` (`--release` for an optimized build)
- `cargo run`: build and run `src/main.rs`
- `cargo test`: build and run every `#[test]` function

This crate has a library (`src/lib.rs`, the exercises) and a small binary (`src/main.rs`) that uses it.

### Variables

Variables are immutable unless declared with `mut`:

```rust
let x = 5;
// x = 6;        // error[E0384]: cannot assign twice to immutable variable `x`

let mut y = 5;
y += 1;
println!("{y}"); // 6
```

### Shadowing

`let` with an existing name creates a new variable. The new one may have a different type:

```rust
let spaces = "   ";
let spaces = spaces.len();
println!("{spaces}"); // 3
```

`x = …` changes a variable; `let x = …` shadows it.

### Types

- Integers: `i8`–`i128`, `u8`–`u128`, `isize`, `usize`. The default is `i32`.
- Floats: `f32`, `f64`. The default is `f64`.
- Also `bool`, `char`, and `&'static str`, the type of a string literal (lesson 04).

```rust
let count = 5;     // i32
let total = 1.5;   // f64
let ok = true;     // bool
let label = "hot"; // &'static str
```

### No implicit conversion

Rust never converts between number types for you:

```rust
let n = 5;
// let x = n + 1.5;         // error[E0277]: cannot add a float to an integer
let x = f64::from(n) + 1.5;
println!("{x}");            // 6.5
```

`f64::from(n)` is lossless. `n as f64` also works but may lose data.

### Integer division

Dividing two integers drops the remainder:

```rust
println!("{}", 9 / 5);     // 1
println!("{}", 9.0 / 5.0); // 1.8
```

In PHP `9 / 5` is `1.8`; Rust's integer `/` behaves like `intdiv(9, 5)`.

### Functions return the last expression

A block's value is its last expression, written **without** a semicolon. Use `return` only to exit early.

```rust
fn double(n: i32) -> i32 {
    n * 2
}

println!("{}", double(4)); // 8
```

### `if` is an expression

`if`/`else` produces a value, so Rust needs no ternary operator:

```rust
let n = -3;
let sign = if n < 0 { "negative" } else { "non-negative" };
println!("{sign}"); // negative
```

### `for` and `while`

`for` loops over a range. `1..4` excludes the end; `1..=3` includes it.

```rust
for n in 1..=3 {
    print!("{n} "); // 1 2 3
}

let mut n = 3;
while n > 0 {
    print!("{n} "); // 3 2 1
    n -= 1;
}
```

### `loop` and `break`

`loop` repeats until `break`. `break value` makes the loop return that value:

```rust
let mut n = 1;
let found = loop {
    if n * n > 20 {
        break n;
    }
    n += 1;
};
println!("{found}"); // 5
```

Only `loop` can return a value; `for` and `while` always give `()`.

### Tests

A function marked `#[test]` is a test. `assert_eq!` compares two values; `assert!` checks a condition.

```rust
#[test]
fn adds() {
    assert_eq!(1 + 1, 2);
    assert!(2 > 1, "optional message");
}
```

Floats are inexact (`0.1 + 0.2` is `0.30000000000000004`), so this lesson's tests compare them with a tolerance (`assert_close`).

## Exercise

1. `celsius_to_fahrenheit`, `fahrenheit_to_celsius`: use float literals (`9.0`, not `9`).
2. `celsius_to_fahrenheit_whole`: integers only. Order matters: `37 * 9 / 5` ≠ `37 * (9 / 5)`.
3. `round_to`: shadow the intermediate value. `f64::round` rounds half away from zero, like PHP's `round()`.
4. `describe`: one `if`/`else if` expression, no `return`.
5. `average_fahrenheit`: a `for` loop over an inclusive range, with `let mut` accumulators.
6. `first_celsius_reaching`: `loop` with `break value`.

Then `cargo run` prints the conversion table.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `37 * (9 / 5)` gives the wrong answer
  - why `describe` needs no `return`
  - the difference between `let x = …` and `x = …` on an existing variable
