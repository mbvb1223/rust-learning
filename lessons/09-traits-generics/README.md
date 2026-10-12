# Lesson 09 — Practical traits and generics

Build plain-text and CSV report formatters behind one trait, then share the code that uses them.

## Run

Inside the container:

```bash
cd /workspace/lessons/09-traits-generics
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- csv
```

Unused-variable warnings (and `unused mut` on `with_row`) are expected until you implement the stubs. Don't run `cargo fix`.

## Notes

### Traits

A trait lists methods a type must have. It can also provide default methods that call the required ones:

```rust
trait Animal {
    fn name(&self) -> &'static str; // required

    fn greeting(&self) -> String {  // provided: implementors may override it
        format!("Hello, {}!", self.name())
    }
}

struct Cat;

impl Animal for Cat {
    fn name(&self) -> &'static str {
        "cat"
    }
}

println!("{}", Cat.greeting()); // Hello, cat!
```

A Rust trait is like a PHP interface with default methods. It's not like a PHP `trait`, which is a copy-paste mixin.

- `impl Animal for Cat` is a separate block, so you can add it to a type later.
- Trait methods are only callable where the trait is imported, e.g. `use traits_generics::ReportFormatter;`.

### Generics and bounds

A generic function works for any type that meets its bounds:

```rust
use std::fmt::Display;

fn bracket<T: Display>(value: T) -> String {
    format!("[{value}]")
}

println!("{}", bracket(42));    // [42]
println!("{}", bracket("hey")); // [hey]
```

- The body may only use what the bounds promise. There is no duck typing.
- Long bounds go in a `where` clause: `where T: Display + Clone`.
- The compiler builds a separate copy of the function for each type you call it with, so calls are as fast as normal ones.

PHPStan's `@template T of Formatter` is the closest PHP match, but Rust checks it in the compiler.

### `impl Trait`

In argument position, `impl Trait` is shorthand for a generic:

```rust
fn new(title: impl Into<String>) -> Self // same as fn new<T: Into<String>>(title: T)
```

In return position, it means "some type that implements this trait":

```rust
fn evens(numbers: &[i32]) -> impl Iterator<Item = &i32> {
    numbers.iter().filter(|n| **n % 2 == 0)
}

let numbers = vec![1, 2, 3, 4];
let found: Vec<_> = evens(&numbers).collect();
println!("{found:?}"); // [2, 4]
```

- It must be **one** concrete type. You can't return `PlainText` from one `match` arm and `Csv` from another; use an enum and `match` instead.
- A returned closure that uses a local variable needs `move`, because it outlives the call.

### `Display`

`Display` controls how `{}` prints your type:

```rust
use std::fmt;

struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

let p = Point { x: 1, y: 2 };
println!("{p}");          // (1, 2)
let text = p.to_string(); // "(1, 2)"
```

- `Display` gives you `.to_string()` for free.
- `Debug` (`{:?}`) is for developers and can be derived; `Display` is for users and is written by hand.

PHP's `__toString()` is the nearest match.

### `Default`

`#[derive(Default)]` gives every field its own default: `false`, `0`, `""`, an empty `Vec`.

```rust
#[derive(Debug, Default)]
struct Options {
    verbose: bool,
    depth: u32,
}

let o = Options { depth: 3, ..Options::default() };
println!("{o:?}"); // Options { verbose: false, depth: 3 }
```

- `..Options::default()` fills in the fields you didn't write.
- Write `impl Default` by hand when the field defaults are wrong, such as a `,` delimiter.
- On an enum, mark one variant `#[default]`.

### Conversions

- `From<T> for U`: `U::from(t)`, and `t.into()` for free
- `TryFrom<T> for U`: a conversion that can fail; returns `Result`
- `FromStr for U`: parses text; enables `"csv".parse::<U>()`

```rust
let s: String = "hi".into();
let byte = u8::try_from(300_i32);   // Err(…): 300 doesn't fit in a u8
let n: i32 = "42".parse().unwrap(); // 42
```

- Implement `From`, not `Into`.
- An `impl Into<String>` parameter accepts both `&str` and `String`.

PHP's backed enums have `from()` and `tryFrom()`; Rust returns the error as a value instead of throwing.

### Derives

A derive writes a trait impl for you, if every field implements that trait too:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

let a = Point { x: 1, y: 2 };
let b = a;              // copied, not moved
println!("{}", a == b); // true
```

- `Debug`: `{:?}`; needed by `assert_eq!`
- `Clone`: `.clone()`
- `Copy`: copies instead of moves; no `String` or `Vec` fields allowed
- `PartialEq`, `Eq`: `==`
- `Hash`: lets the type be a `HashMap` key or `HashSet` member; derive it with `Eq`
- `Default`: `T::default()`

## Exercise

Implement in this order:

1. `From<(L, usize)> for Row`: `label.into()`.
2. `Report::new`, `with_row`, `total`: `with_row` takes `mut self`, pushes, and returns `self`; `total` is `iter().map(…).sum()`.
3. `Report::rows_at_least`: replace the whole `todo!() as …` line with `self.rows.iter().filter(…)`. Leave out `move` first and read the error.
4. `Display for Report`: `write!(f, …)`. The tests call `.to_string()`.
5. `join_display`: map each item to a `String`, collect into a `Vec<String>`, then `.join(separator)`.
6. `Display for FormatKind`, then `FromStr` (`str::eq_ignore_ascii_case`), then `TryFrom<&str>` via `s.parse()`, then `Display for ParseFormatError` with `{:?}` and `join_display(&FormatKind::ALL, ", ")`.
7. The trait's provided methods `file_extension` and `file_name`. Then `cargo test --test custom_formatter downstream` passes with no code of its own.
8. `PlainText::format`: collect `(label, count)` pairs plus `("total", report.total())` when shown, take both widths from that list (`chars().count()`, `count.to_string().len()`), then `format!("{label:<width$} {count:>digits$}")` and `"=".repeat(n)`.
9. `csv_field`: `field.contains([delimiter, '"', '\r', '\n'])`, then `str::replace` to double the quotes.
10. `Csv`: `Default`, `format`, and the `file_extension` override.
11. `export`, then `export_as`, calling `export` from each `match` arm.

Then try `cargo run -- csv`, `cargo run`, and `cargo run -- xml`.

Experiments (undo each afterwards):

- Replace `impl Default for Csv` with `#[derive(Default)]`. Why does it compile, and which tests fail?
- Remove `Hash` from `FormatKind`'s derives and read the error.
- Write `fn pick(kind: FormatKind) -> impl ReportFormatter` returning `PlainText::default()` or `Csv::default()` from a `match`, and read the error.

Optional: add `formatter_for(kind) -> Box<dyn ReportFormatter>` and pass `&*formatter_for(kind)` to `export`; make it compile with `F: ReportFormatter + ?Sized`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - how a Rust trait differs from a PHP interface, and from a PHP trait
  - why the `pick` experiment doesn't compile as `-> impl ReportFormatter`, and how `export_as` avoids the problem
  - why implementing `From` is enough to call `.into()`
