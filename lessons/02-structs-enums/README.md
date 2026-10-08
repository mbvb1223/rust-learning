# Lesson 02 — Model data

Build a money type and an order state machine: implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

From the repository root:

```bash
docker compose run --rm -w /workspace/lessons/02-structs-enums rust cargo test
docker compose run --rm -w /workspace/lessons/02-structs-enums rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/02-structs-enums rust cargo clippy --all-targets -- -D warnings
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

## Notes

### Structs and `impl`

```rust
pub struct Money {
    minor: i64,          // private: only code in this module can read it
    currency: Currency,
}

impl Money {
    pub fn new(minor: i64, currency: Currency) -> Self { // associated function ≈ static named constructor
        Self { minor, currency }                         // field init shorthand
    }

    pub fn minor(&self) -> i64 { self.minor }            // method; `&self` ≈ read-only `$this`
}

let price = Money::new(1250, Currency::Eur);             // `::` for associated functions
price.minor();                                           // `.` for methods
```

- A struct holds data only; behaviour goes in `impl` blocks. There is no inheritance and no constructor keyword.
- A struct is a **value**, not an object handle. Lesson 03 explains what happens when you pass one around.
- `self` (by value) vs `&self` (borrowed) is covered in lesson 03. Here every type is `Copy`, so both work.

### Enums with data

```rust
pub enum OrderState {
    Pending,
    Paid { amount: Money },
    Shipped { amount: Money, tracking: u64 },
}
```

PHP 8.1 enum cases can't carry per-case data. The closest PHP model is a sealed abstract class with one final subclass per case. Rust enums hold different data per variant, and the compiler knows every variant.

### `match`

```rust
match state {
    OrderState::Pending => "pending",
    OrderState::Paid { amount } => …,                 // destructure fields
    OrderState::Shipped { tracking, .. } => …,        // `..` ignores the rest
}

match state {
    OrderState::Pending if amount.minor() > 0 => …,   // guard
    OrderState::Pending | OrderState::Paid { .. } => …,
    _ => …,                                           // catch-all
}
```

- `match` is **exhaustive at compile time**: a missing variant is a compile error. PHP's `match` throws `UnhandledMatchError` at runtime.
- On your own enums, avoid `_ =>` where every case matters. Without it, adding a variant makes the compiler point at every `match` that needs updating.

### `Option<T>` instead of `null`

```rust
let maybe: Option<Money> = state.amount();   // Some(money) or None

if let Some(amount) = maybe {                // run one branch only on a match
    println!("{}", amount.format());
}

let Some(amount) = maybe else {              // let-else: bind or leave early
    return None;
};
```

- `?Money` + `null` becomes `Option<Money>`. You can't use the inner value without handling `None`.
- Inside a function that returns `Option`, `value?` returns `None` early. Lesson 05 covers `?` in depth.
- Trap: PHP's `??` is lazy, but `opt.unwrap_or(expensive())` always calls `expensive()`. Use `opt.unwrap_or_else(|| expensive())`.

### Derives

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

- `Debug` enables `{:?}` printing; `assert_eq!` needs it.
- `PartialEq`/`Eq` enable `==`.
- `Clone`/`Copy` let a value be used again after passing it by value. Lesson 03 explains them.

### Money in minor units

Never store money as a float (`0.1 + 0.2 != 0.3`). Store integer minor units (cents), like `moneyphp/money` or `brick/money` do. Arithmetic uses `i64::checked_add` / `checked_sub` / `checked_mul`, which return `None` on overflow instead of panicking.

## Exercise

Part A — `Money`:

1. `Currency::code` and `Currency::decimals`: `match`, combining arms with `|`.
2. `Money::new`, `minor`, `currency`.
3. `Money::from_major`: `10_i64.pow(decimals)` and `checked_mul`.
4. `checked_add` / `checked_sub`: `None` on mixed currencies or overflow.
5. `format`: `"12.50 EUR"`, `"-0.05 EUR"`, `"1250 JPY"`.
   - Watch the sign of amounts under one unit.
   - `i64::unsigned_abs()` helps.
   - `format!("{n:0width$}", width = 2)` zero-pads.

Part B — `OrderState`:

6. `pay`, `ship`, `deliver`, `cancel`: return `None` for any transition the type's doc comment doesn't allow. Use guards and `|` patterns.
7. `amount`: one `match` with combined arms.
8. `tracking`: `if let`.
9. `label`: an exhaustive `match` with no `_` arm.
10. `refund`: start with `let … else`.

Optional extension: add a `Refunded { amount: Money }` variant and let the compiler errors guide every `match` that needs updating. Decide which transitions lead to it, and add tests.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `label` should have no `_` arm
  - why `Money` stores `i64` minor units
  - what `if let` and `let … else` each save you compared with a full `match`
