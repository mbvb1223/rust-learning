# Lesson 03 — Ownership and borrowing

Fix seven borrow-checker errors in `src/broken.rs`, then build a shopping cart in `src/lib.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/03-ownership-borrowing
cargo test
cargo check --features broken
cargo test --features broken
cargo fmt --check
cargo clippy --all-targets --features broken -- -D warnings
```

`src/broken.rs` only compiles with `--features broken`, so plain `cargo test` works while it's still broken. Don't run `cargo fix`.

## Notes

### Ownership

Every value has one owner. When the owner goes out of scope, the value is dropped (freed).

Assigning a `String` **moves** it. The old variable can't be used anymore:

```rust
let a = String::from("milk");
let b = a;
println!("{b}");    // milk
// println!("{a}"); // error[E0382]: borrow of moved value: `a`
```

PHP would copy the string here. Rust has no garbage collector, so exactly one variable owns the value and frees it.

Passing a value to a function moves it too:

```rust
fn consume(s: String) {}

let s = String::from("bread");
consume(s);
// println!("{s}"); // error: `s` was moved into `consume`
```

### Copy and clone

Simple types (integers, floats, `bool`, `char`) are **copied** instead of moved:

```rust
let x = 5;
let y = x;
println!("{x} {y}"); // 5 5
```

To copy a `String` or `Vec`, call `.clone()`:

```rust
let a = String::from("milk");
let b = a.clone();
println!("{a} {b}"); // milk milk
```

Clone only when you really need a second copy. Borrowing is usually the better fix.

### Scope and drop

A value is dropped at the end of its scope. Variables in the same scope are dropped in **reverse** order.

```rust
{
    let a = String::from("a");
    let b = String::from("b");
} // b dropped, then a
```

- `drop(x)` drops a value early.
- A temporary (a value not stored in a variable) is dropped at the end of its statement.
- `let _ = value;` drops at once; `let _guard = value;` keeps it until the scope ends.

### Borrowing

`&value` lends a value without moving it. The owner keeps it:

```rust
fn length(s: &String) -> usize {
    s.len()
}

let s = String::from("milk");
println!("{}", length(&s)); // 4
println!("{s}");            // milk
```

`&mut value` lends it with permission to change it:

```rust
fn add_one(n: &mut i32) {
    *n += 1;
}

let mut count = 1;
add_one(&mut count);
println!("{count}"); // 2
```

### The borrowing rule

At any time a value has **either** many `&` borrows **or** one `&mut` borrow, never both.

```rust
let mut v = vec![1, 2];
let first = &v[0];
v.push(3); // error[E0502]: `v` is still borrowed by `first`
println!("{first}");
```

`push` may move the vector's data, so `first` would point at freed memory. A borrow ends at its last use: move the `println!` above the `push` and it compiles.

PHP's `foreach` iterates over a copy, so changing an array inside its own loop works there. In Rust it's a compile error.

### Loops and ownership

```rust
for item in &v {}     // borrows: item is &T, v is still usable
for item in &mut v {} // borrows mutably: item is &mut T
for item in v {}      // moves: item is T, v is gone afterwards
```

### Method receivers

```rust
impl Cart {
    fn items(&self) {}     // read only
    fn add(&mut self) {}   // may change the cart
    fn into_items(self) {} // takes the cart; the caller can't use it afterwards
}
```

### Error codes you'll meet

- E0382: used a value after it was moved
- E0499: two `&mut` borrows at once
- E0502: `&mut` borrow while a `&` borrow is still in use
- E0505: moved a value while it was borrowed
- E0507: moved out of something you only borrowed
- E0515: returned a reference to a local variable
- E0596: changed a variable that isn't `mut`

In the error output, the `^^^` label marks the refused operation and "later used here" marks what keeps the borrow alive. The fix is usually between them. `rustc --explain E0502` explains any code.

## Exercise

Part A — `src/broken.rs`:

1. Run `cargo check --features broken`. You'll see seven errors, one per function.
2. Fix them one at a time. Keep each doc comment's contract. You may change a signature, but not the tests. Avoid `.clone()`.
3. Fill in the table:

| Function | Error code | Your fix, and why it is correct |
|---|---|---|
| `sorted` | | |
| `shout_all` | | |
| `customer_name` | | |
| `normalize` | | |
| `record` | | |
| `append_doubles` | | |
| `transfer` | | |

4. `cargo test --features broken broken::` runs only Part A's tests.

Part B — `Cart` in `src/lib.rs`:

5. `LineItem::new` and `subtotal_cents`: `.to_string()` turns a `&str` into a `String`; `i64::from(quantity)` widens the `u32`.
6. `Cart::new` and `items`: return `&self.items`.
7. `find`: `self.items.iter().find(|item| item.sku == sku)`. `|item| …` is a closure, like PHP's `fn($item) => …`.
8. `add`: ignore quantity 0. Find an existing line with `iter_mut()` and bump its quantity, otherwise `push`.
9. `remove`: write a private `fn position(&self, sku: &str) -> Option<usize>` with `iter().position(…)`, then use `Vec::remove(index)`.
10. `set_quantity`: `position`, then set `self.items[index].quantity` or remove the line.
11. `total_cents`: a `for` loop over `&self.items`.
12. `merge`: `for item in other.items`, calling `add` for each.
13. `into_items`: return the field.

Optional: add `apply_discount(&mut self, percent: u32)` using `for item in &mut self.items`, with tests.

## Done when

- `cargo test`, `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --features broken -- -D warnings` pass.
- The Part A table is filled in.
- You can explain:
  - why `let b = a;` makes `a` unusable for a `String` but not for an `i64`
  - why `find` returns `Option<&LineItem>`, and what you can't do to the cart while you hold it
  - the difference between `for item in v`, `&v`, and `&mut v`
