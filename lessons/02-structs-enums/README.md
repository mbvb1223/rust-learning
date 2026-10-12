# Lesson 02 — Model data

Build a money type and an order state machine in `src/lib.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/02-structs-enums
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Don't run `cargo fix`.

## Notes

### Structs

A struct groups named fields. Fields are private to their module unless marked `pub`.

```rust
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };
println!("{}", p.x); // 1
```

### Methods with `impl`

Behaviour goes in an `impl` block. A function without `self` is called with `::`, like a static constructor; a method takes `&self`, like a read-only `$this`.

```rust
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y } // shorthand for Self { x: x, y: y }
    }

    fn sum(&self) -> i32 {
        self.x + self.y
    }
}

let p = Point::new(1, 2);
println!("{}", p.sum()); // 3
```

- No inheritance and no constructor keyword: `new` is just a convention.
- Some stubs take `self`, others `&self`. Both work here because every type in this lesson is `Copy` (lesson 03).

### Enums

An enum has a fixed set of variants, and each variant can carry its own data:

```rust
enum Shape {
    Square { side: f64 },
    Rect { w: f64, h: f64 },
    Empty,
}

let s = Shape::Rect { w: 2.0, h: 3.0 };
```

PHP enum cases can't carry per-case data; the closest PHP model is a sealed class hierarchy.

### `match`

`match` runs the first arm whose pattern fits, and can pull fields out of the variant:

```rust
let area = match s {
    Shape::Square { side } => side * side,
    Shape::Rect { w, h } => w * h,
    Shape::Empty => 0.0,
};
println!("{area}"); // 6
```

`match` is exhaustive: leave out a variant and it doesn't compile. PHP's `match` only throws `UnhandledMatchError` at runtime.

### More patterns

```rust
let label = match s {
    Shape::Square { side } if side > 10.0 => "big square",
    Shape::Square { .. } | Shape::Rect { .. } => "box",
    _ => "nothing",
};
println!("{label}"); // box
```

- `if …` after a pattern is a guard.
- `|` combines patterns into one arm.
- `..` ignores the remaining fields.
- `_` matches anything else. Avoid it on your own enums: without it, a new variant makes the compiler point at every `match` to update.

### `Option` instead of `null`

`Option<T>` is either `Some(value)` or `None`. It replaces `?Type` and `null`, and you can't use the value without handling `None`.

```rust
let found: Option<i32> = Some(5);
match found {
    Some(n) => println!("got {n}"), // got 5
    None => println!("nothing"),
}
```

### `if let` and `let … else`

`if let` runs a block only when one pattern matches:

```rust
if let Some(n) = found {
    println!("{n}"); // 5
}
```

`let … else` binds the value or leaves early:

```rust
fn double(found: Option<i32>) -> Option<i32> {
    let Some(n) = found else {
        return None;
    };
    Some(n * 2)
}

println!("{:?}", double(Some(5))); // Some(10)
```

In a function that returns `Option`, `let n = found?;` does the same in one step (lesson 05).

### Derives

`#[derive(…)]` adds standard behaviour to a type:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };
println!("{p:?}");                        // Point { x: 1, y: 2 }
println!("{}", p == Point { x: 1, y: 2 }); // true
```

- `Debug`: print with `{:?}`. `assert_eq!` needs it.
- `PartialEq`, `Eq`: compare with `==`.
- `Clone`, `Copy`: reuse a value after passing it by value (lesson 03).

### Money in minor units

Never store money as a float. Store whole cents in an integer, like `moneyphp/money` does.

```rust
println!("{}", 0.1 + 0.2); // 0.30000000000000004
let price: i64 = 1250;     // 12.50 EUR
```

### Checked arithmetic

`checked_add`, `checked_sub`, and `checked_mul` return `None` on overflow instead of panicking:

```rust
println!("{:?}", 5_i64.checked_add(1));    // Some(6)
println!("{:?}", i64::MAX.checked_add(1)); // None
```

### Zero-padding

`format!` builds a `String`, like `sprintf`. `{n:0width$}` pads `n` with zeros to `width` digits:

```rust
let n = 5;
println!("{n:02}"); // 05

let s = format!("{n:0width$}", width = 3);
println!("{s}");    // 005
```

## Exercise

Part A — `Money`:

1. `Currency::code` and `Currency::decimals`: `match`, combining arms with `|`.
2. `Money::new`, `minor`, `currency`.
3. `Money::from_major`: `10_i64.pow(decimals)` and `checked_mul`.
4. `checked_add` / `checked_sub`: `None` on mixed currencies or overflow.
5. `format`: `"12.50 EUR"`, `"-0.05 EUR"`, `"1250 JPY"`. Watch the sign of amounts under one unit; `i64::unsigned_abs()` helps.

Part B — `OrderState`:

6. `pay`, `ship`, `deliver`, `cancel`: `None` for any transition the type's doc comment doesn't allow. Use guards and `|` patterns.
7. `amount`: one `match` with combined arms.
8. `tracking`: `if let`.
9. `label`: an exhaustive `match` with no `_` arm.
10. `refund`: start with `let … else`.

Optional: add a `Refunded { amount: Money }` variant, let the compiler errors show every `match` to update, and add tests.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `label` should have no `_` arm
  - why `Money` stores `i64` minor units
  - what `if let` and `let … else` each save you compared with a full `match`
