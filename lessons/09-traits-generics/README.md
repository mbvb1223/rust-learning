# Lesson 09 — Practical traits and generics

Build plain-text and CSV report formatters behind one trait, then share the code that uses them: implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

From the repository root:

```bash
docker compose run --rm -w /workspace/lessons/09-traits-generics rust cargo test
docker compose run --rm -w /workspace/lessons/09-traits-generics rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/09-traits-generics rust cargo clippy --all-targets -- -D warnings
docker compose run --rm -w /workspace/lessons/09-traits-generics rust cargo run -- csv
```

Until a function is implemented, its parameters show `unused variable` warnings (`with_row` also shows `unused mut`). Don't run `cargo fix`.

`tests/custom_formatter.rs` is an integration test: a separate crate that sees only the public API, like another Composer package implementing your interface. `src/main.rs` is a small CLI that uses the library.

## Notes

### Traits are interfaces with default methods

```rust
pub trait ReportFormatter {
    fn format(&self, report: &Report) -> String;    // required

    fn file_extension(&self) -> &'static str {      // provided: implementors may override it
        "txt"
    }
}

impl ReportFormatter for Csv {
    fn format(&self, report: &Report) -> String { … }
    fn file_extension(&self) -> &'static str { "csv" }
}
```

| PHP | Rust |
|---|---|
| `interface Formatter` | `trait ReportFormatter` |
| `class Csv implements Formatter` | `impl ReportFormatter for Csv`: a separate block, can be added later |
| abstract base class with shared methods | provided methods in the trait itself; no fields, no inheritance |
| `trait Loggable` + `use Loggable;` | no equivalent: a PHP trait is a copy-paste mixin, not a type, and you can't type-hint it |
| `Stringable` / `__toString()` | `impl Display` |

- A provided method can call any other trait method. `file_name` calls `self.file_extension()`, so overriding the extension also fixes the file name.
- You can implement your trait for types you don't own (`impl ReportFormatter for String`) and std traits for your types (`impl Display for Report`), but not a foreign trait for a foreign type: either the trait or the type must be local (the *orphan rule*).
- A trait's methods are only callable where the trait is imported. `PlainText::default().format(&r)` in another crate needs `use traits_generics::ReportFormatter;`, otherwise: "no method named `format` found". In PHP, an object's methods are always callable.

### Generics and bounds

```rust
pub fn export<F: ReportFormatter>(formatter: &F, report: &Report, stem: &str) -> Export { … }

fn describe<T>(item: &T) -> String
where
    T: Display + Clone,    // several bounds; `where` keeps long signatures readable
{ … }
```

- The body may only use what the bounds promise. `formatter.format(…)` compiles because `F: ReportFormatter` guarantees it exists. There is no duck typing.
- **Monomorphization:** the compiler generates a separate copy of `export` for each type it's called with (`export::<PlainText>`, `export::<Csv>`). Calls are resolved at compile time and can be inlined; the cost is compile time and binary size.
- PHP comparison: a `Formatter $f` type hint is checked at runtime on every call. A PHPStan `@template T of Formatter` is checked by a static analyser and does nothing at runtime. Rust bounds are checked at compile time and produce specialized code.
- Impls can be generic too: `impl<L: Into<String>> From<(L, usize)> for Row` is one impl covering `(&str, usize)`, `(String, usize)`, and every other `L: Into<String>`.

### `impl Trait`

Argument position: shorthand for an anonymous generic parameter.

```rust
pub fn new(title: impl Into<String>) -> Self                   // ≈ fn new<T: Into<String>>(title: T)
pub fn join_display(items: &[impl Display], separator: &str) -> String
```

The caller can't name that type: `join_display::<i32>(…)` doesn't compile. Pass a typed value instead (`let none: &[i32] = &[];`).

Return position: "one concrete type that implements this trait, chosen by the function".

```rust
pub fn rows_at_least(&self, min: usize) -> impl Iterator<Item = &Row> {
    self.rows.iter().filter(move |row| …)
}
```

- ≈ a PHP `: iterable` return type, but resolved at compile time with no boxing. The caller can only use `Iterator` methods.
- It must be **one** type. A function returning `PlainText` from one `match` arm and `Csv` from another can't be `-> impl ReportFormatter`: use an enum + `match` (`export_as`) or a trait object (below).
- Since edition 2024 the hidden type captures every lifetime in scope. `rows_at_least` would compile in 2021 too, because `Item = &Row` names `self`'s lifetime. Older code needed `+ '_` when the bound doesn't name it, e.g. `fn counts(&self) -> impl Iterator<Item = usize> + '_ { self.rows.iter().map(|r| r.count) }`.
- The closure still needs `move`: it outlives the call, so it can't borrow the local `min`.
- The stub reads `todo!() as std::iter::Empty<&Row>` because `todo!()` has type `!`, which doesn't implement `Iterator`, so it can't be the hidden type. Replace the whole line.

### `Display` and `Default`

```rust
impl fmt::Display for FormatKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "…")    // or f.write_str("…")
    }
}
```

- `Debug` (`{:?}`) is for developers and can be derived. `Display` (`{}`) is for users and has no built-in derive.
- `Display` gives you `.to_string()`: std implements `ToString` for every `T: Display`, a *blanket impl*.
- `write!(f, …)` ignores width flags such as `{:<8}` applied to your value. Use `f.pad("text")` if callers should be able to align it.

```rust
#[derive(Default)]                 // each field's own default: false, 0, "", empty Vec
pub struct PlainText { pub show_total: bool }

impl Default for Csv {             // hand-written: derived '\0' and false would be wrong
    fn default() -> Self { … }
}

let csv = Csv { header: false, ..Csv::default() };    // struct update: override some fields
```

- On an enum, `#[derive(Default)]` needs one variant marked `#[default]` (see `FormatKind`).
- Struct update syntax ≈ PHP named arguments with constructor defaults (`new Csv(header: false)`).

### Conversions: `From`, `Into`, `TryFrom`, `FromStr`

| Trait | Use for | You get |
|---|---|---|
| `From<T> for U` | infallible conversion | `U::from(t)`, and `t.into()` via a blanket impl |
| `TryFrom<T> for U` | fallible conversion | `U::try_from(t)` → `Result`, and `t.try_into()` |
| `FromStr for U` | parsing text | `"csv".parse::<U>()` → `Result` |

```rust
let row: Row = ("apples", 3).into();
let byte = u8::try_from(300_i32);          // Err(TryFromIntError(()))
let kind: FormatKind = "csv".parse()?;     // the target type selects the FromStr impl
```

- Implement `From`, not `Into`; `Into` comes for free.
- `impl Into<String>` parameters accept `&str` and `String`: callers write `Report::new("Sales")` without `.to_string()`.
- PHP 8.1 backed enums are the nearest match: `Format::from('xml')` throws `ValueError`, `Format::tryFrom('xml')` returns `null`. `FromStr` returns `Err(ParseFormatError { … })`, a value that says what went wrong.
- `FormatKind` implements both `FromStr` and `TryFrom<&str>` so you practise both. `FromStr` is the convention for text: it enables `.parse()`, and argument parsers such as `clap` (lesson 15) use it. `TryFrom` is the general form, used for conversions between other types (`u8::try_from(300_i32)`).
- Give error types `Display` and `impl std::error::Error`, so they work with `?` and `Box<dyn Error>` (lesson 05).

### Derives

| Derive | Gives | Requirement / gotcha |
|---|---|---|
| `Debug` | `{:?}`; needed by `assert_eq!` | — |
| `Clone` | `.clone()` | — |
| `Copy` | implicit copies instead of moves | also derive `Clone`; all fields `Copy`: no `String`/`Vec`, so `Report` can't be `Copy` |
| `PartialEq`, `Eq` | `==` | `Eq` promises every value equals itself; an `f64` field (NaN) rules it out |
| `Hash` | `HashMap` key, `HashSet` member | derive with `Eq`: equal values must hash equally |
| `Default` | `T::default()` | every field must implement `Default` |
| `PartialOrd`, `Ord` | `<`, `.sort()` | structs compare field by field in declaration order; enums by variant order first |

A derive compiles only if every field implements that trait. Each derive is public API, and removing one later breaks callers, so derive what the type needs.

### Trait objects (advanced, brief)

```rust
fn formatter_for(kind: FormatKind) -> Box<dyn ReportFormatter> {
    match kind {
        FormatKind::Text => Box::new(PlainText::default()),
        FormatKind::Csv => Box::new(Csv::default()),
    }
}
```

- `dyn ReportFormatter` means dynamic dispatch: one compiled function, with the method looked up in a vtable at runtime. It's the closest match to PHP interface calls and allows mixed collections: `Vec<Box<dyn ReportFormatter>>`.
- Choose by the set of types. A closed, known set → enum + `match`. An open set (plugins, other crates) → generics, or `dyn` when the type is only known at runtime.
- A trait with generic methods can't be used as `dyn` (it must be *dyn compatible*). The roadmap's advanced topics cover trait objects, associated types, and the orphan rule.

## Exercise

Implement in this order:

1. `From<(L, usize)> for Row`: `label.into()`. `("apples", 3).into()` now works too.
2. `Report::new`, `with_row`, `total`. `with_row` takes `mut self`, pushes, and returns `self`. `total`: `iter().map(…).sum()`.
3. `Report::rows_at_least`: `self.rows.iter().filter(…)`. Leave out `move` first and read the error.
4. `Display for Report`: `write!(f, …)`. The tests call `.to_string()`, which you never write.
5. `join_display`: map each item to a `String`, collect into a `Vec<String>`, then `.join(separator)`.
6. `Display for FormatKind`, then `FromStr` (`str::eq_ignore_ascii_case`), then `TryFrom<&str>` by delegating to `s.parse()`, then `Display for ParseFormatError` with `{:?}` and `join_display(&FormatKind::ALL, ", ")`.
7. The trait's provided methods `file_extension` and `file_name`. The `TitleOnly` integration test now passes with no code of its own: `cargo test --test custom_formatter downstream` (plain `cargo test` runs `tests/` only once every unit test passes).
8. `PlainText::format`:
   - Collect `(label, count)` pairs, push `("total", report.total())` when shown, then compute both widths from that list.
   - `"=".repeat(n)`, `str::chars().count()`, `count.to_string().len()`.
   - `format!("{label:<width$} {count:>digits$}")` pads by `char`s, not bytes.
9. `csv_field`: `field.contains([delimiter, '"', '\r', '\n'])` and `str::replace`.
   - Quoting follows RFC 4180, but records end with `\n` rather than the RFC's CRLF (most parsers accept both).
   - Unlike PHP's `fputcsv`, it doesn't quote fields just for containing spaces, and it never uses a `\` escape.
10. `Csv`: `Default`, `format`, and the `file_extension` override.
11. `export`, then `export_as`, calling `export` from each `match` arm.

Then try `cargo run -- csv`, `cargo run`, and `cargo run -- xml`.

Experiments (undo each afterwards):

- Replace `impl Default for Csv` with `#[derive(Default)]`. Why does it compile, and which tests fail?
- Remove `Hash` from `FormatKind`'s derives and read the error.
- Write `fn pick(kind: FormatKind) -> impl ReportFormatter` returning `PlainText::default()` or `Csv::default()` from a `match`, and read the error.

Optional extension: add `formatter_for` (above) and an `export_dyn(kind, report, stem)` that calls `export(&*formatter_for(kind), report, stem)`. It fails: generic parameters are implicitly `Sized`, and `dyn ReportFormatter` isn't. Relax the bound to `F: ReportFormatter + ?Sized`, then compare `export_dyn` with `export_as`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - how a Rust trait differs from a PHP interface, and from a PHP trait
  - what monomorphization produces for `export`, and how a `Box<dyn ReportFormatter>` call is dispatched instead
  - why the `pick` experiment doesn't compile as `-> impl ReportFormatter`, and how `export_as` avoids the problem
  - why `Csv` has a hand-written `Default` but `PlainText` derives it
  - why implementing `From` is enough to call `.into()`
  - why `FormatKind` can derive `Copy` and `Hash` but `Report` can't derive `Copy`
