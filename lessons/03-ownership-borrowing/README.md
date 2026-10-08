# Lesson 03 — Ownership and borrowing

Fix seven borrow-checker errors in `src/broken.rs`, then build a shopping cart: implement every `todo!()` in `src/lib.rs` until the tests pass.

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

`src/broken.rs` is compiled only with `--features broken` (the `[features]` table in `Cargo.toml` plus `#[cfg(feature = "broken")]` in `lib.rs`). Plain `cargo test` runs Part B even while Part A doesn't compile.

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

## Notes

### Ownership and moves

Every value has exactly one owner: a variable, a struct field, a slot in a `Vec`. When the owner goes out of scope, the value is dropped. Assigning or passing a non-`Copy` value **moves** it; the old name becomes unusable at compile time.

```rust
let a = String::from("milk");
let b = a;                 // move: `b` owns the string now
println!("{a}");           // error[E0382]: borrow of moved value: `a`

fn consume(s: String) {}   // a by-value parameter takes ownership
consume(b);                // `b` moves into the call and is dropped when `consume` returns
```

| | PHP | Rust |
|---|---|---|
| `$b = $a` (string, array) | copy-on-write copy; both usable | move; `a` unusable |
| `$b = $a` (object) | two handles to one object, refcount 2 | move; still exactly one owner |
| `f($a)` | callee gets a copy or a handle; caller keeps `$a` | callee owns it; pass `&a` to lend it instead |
| Freed when | refcount hits 0, or the cycle collector runs | the owner's scope ends; decided at compile time |

Where the PHP picture breaks: Rust keeps no reference count at run time and has no garbage collector. The compiler knows the single owner, so it inserts the cleanup itself. Shared ownership (`Rc`, `Arc`) is opt-in; `Arc` arrives in lesson 12.

### Stack, heap, and why a move invalidates

A `String` is three machine words — pointer, capacity, length — and the bytes live on the heap. `Vec<T>` has the same shape.

```text
stack                      heap
a: [ptr | cap 4 | len 4] ──► m i l k
```

`let b = a;` copies the three words, never the heap bytes. If `a` stayed usable, both would free the same buffer at scope end (a double free). Rust forbids using `a` instead, so a move is a cheap shallow copy plus a compile-time rule.

### `Copy` and `Clone`

- **`Copy`** types are duplicated bit-for-bit, and the original stays usable: integers, floats, `bool`, `char`, shared references `&T`, and tuples/arrays of `Copy` types. Your own type opts in with `#[derive(Clone, Copy)]`, as lesson 02's `Money` did.
- A type that owns heap data (`String`, `Vec`, any struct holding one) can't be `Copy`. **`.clone()`** makes an explicit deep copy.
- PHP's `clone $cart` is shallow: nested objects stay shared unless you write `__clone`. Rust's derived `Clone` clones every field, so a cloned `Cart` shares nothing with the original.
- Clone when you really need a second, independent value. A clone added only to silence the borrow checker usually hides a better fix.

### Scope and drop order

`Noisy(&'static str)` stands for any type whose drop is observable: it prints its name when dropped (a custom `Drop` impl, an advanced topic).

```rust
{
    let a = Noisy("a");
    let b = Noisy("b");
    let c = Noisy("c");
    drop(c);                       // dropped now: `drop` takes ownership and returns nothing
    let _ = Noisy("ignored");      // `_` doesn't bind: dropped immediately
    let n = Noisy("temp").0.len(); // temporary: dropped at the end of this statement
}                                  // then `b`, then `a`
```

- Locals are dropped at the end of their scope in **reverse** declaration order. A struct's fields drop in declaration order; a `Vec` drops its elements first to last.
- A moved-from variable is not dropped again; the new owner is responsible.
- `let _guard = …` keeps the value until scope end; `let _ = …` drops it at once.
- PHP's `__destruct` usually runs at a similar point, but objects in reference cycles wait for the cycle collector. Rust drop points are known at compile time. Rust still doesn't promise that every destructor runs (`std::mem::forget`, leaked `Rc` cycles, aborts), so never rely on `Drop` for memory safety. Custom `Drop` impls are an advanced topic.

### Borrowing: `&T` and `&mut T`

A reference borrows a value without owning it. The rule, checked at compile time:

> At any point, a value has **either** any number of shared borrows `&T` **or** exactly one exclusive borrow `&mut T`. No borrow outlives the owner.

```rust
let mut v = vec![1, 2];
let first = &v[0];   // shared borrow starts
v.push(3);           // needs `&mut v` while `first` is alive
println!("{first}"); // `first` used here, so it was alive at the push
```

`push` may reallocate the buffer and leave `first` pointing at freed memory. Delete the `println!` and it compiles: a borrow lasts until its **last use**, not to the end of the block (non-lexical lifetimes).

- PHP references (`$b = &$a`) are aliases: two names, both may write, no rules. Rust references are checked loans. PHP's copy-on-write means mutating an array inside its own `foreach` is legal there; in Rust it's a compile error.
- `&T` is `Copy`; `&mut T` is not. Method calls borrow automatically: `v.push(3)` is `Vec::push(&mut v, 3)`, and `item.name` reads through `&LineItem` without `*`.
- `for x in v` consumes `v`; `for x in &v` yields `&T`; `for x in &mut v` yields `&mut T`.

### Reading a borrow-checker diagnostic

The snippet above, inside `fn main`, produces:

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let first = &v[0];
  |                  - immutable borrow occurs here
4 |     v.push(3);
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |                ----- immutable borrow later used here
```

Read it in this order:

1. **Headline**: the error code and the operation that was refused.
2. **`^^^` label**: where the refused operation happens.
3. **`---` labels**: where the conflicting borrow or move started, and **"later used here"** — the use that keeps it alive. The fix almost always targets the span between those two labels: end the borrow sooner, don't take it, or take ownership instead.
4. **`help:`** lines: often useful, not always right. In `record`, the suggested clone doesn't compile either.
5. `rustc --explain E0502` prints a long explanation with examples.

| Code | Meaning |
|---|---|
| E0382 | use of a value after it was moved |
| E0499 | two `&mut` borrows of the same value alive at once |
| E0502 | `&mut` borrow while a `&` borrow is alive, or the reverse |
| E0505 | moving a value while it is borrowed |
| E0507 | moving out of something you only borrowed |
| E0515 | returning a reference to a local that is about to be dropped |
| E0596 | `&mut` borrow of a binding not declared `mut` |
| E0716 | borrowing a temporary that is dropped at the end of the statement |

Fix toolbox, in rough order of preference: reorder so a borrow ends before the conflicting use; borrow instead of move (`&v`, `&str` parameters); return an owned value instead of a reference; compute first, then mutate in a second step; index one element at a time instead of holding two `&mut`; declare the binding `mut`; clone when you truly need a second copy.

### Choosing a receiver

| Receiver | Meaning | PHP analogy |
|---|---|---|
| `&self` | shared borrow, read-only | a method that only reads `$this` |
| `&mut self` | exclusive borrow, may mutate | a method that writes `$this->…` |
| `self` | takes ownership; the caller's value is gone, unless the type is `Copy` (lesson 02's `Money`), in which case the method gets a copy | none — PHP can't end an object's life from inside a method |

Why each `Cart` method has its signature:

| Method | Why |
|---|---|
| `items`, `find`, `total_cents` (`&self`) | Read-only. The returned `&[LineItem]` / `Option<&LineItem>` borrows from the cart: no copy, and the cart can't change while the caller holds it. Handing out `&` rather than `&mut` also stops callers breaking the invariants, for example by setting a quantity to 0. |
| `add` (`&mut self`, `item: LineItem`) | Changes the cart in place. The cart must own what it stores, so it takes `item` by value; the caller gives it up, with no clone. |
| `remove` (`&mut self`) → `Option<LineItem>` | Ownership of the removed line moves back to the caller. |
| `set_quantity`, `find`, `remove` (`sku: &str`) | The SKU is only compared, never stored, so borrowing is enough. |
| `merge` (`&mut self`, `other: Cart`) | Consuming `other` lets its lines move into `self` without cloning, and the compiler stops the caller from using the emptied cart afterwards. |
| `into_items` (`self`) | Ends the cart's life and hands back its `Vec`. The `into_` prefix is the std convention for consuming conversions. |

`LineItem::new` takes `&str`, which is convenient with literals but always allocates new `String`s. A constructor taking `String` would let callers hand over an allocation they already own; an `impl Into<String>` parameter accepts both (conversion traits are lesson 09).

## Exercise

Part A — fix the borrow checker (`src/broken.rs`):

1. Run `cargo check --features broken`. You should see seven errors, one per function, all reported together.
2. Fix them one at a time, re-running `cargo check --features broken` after each. Keep every doc-comment contract. You may change a signature (such as a return type) when that is the right fix; don't edit the tests. Prefer fixes without `.clone()`.
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

4. `cargo test --features broken broken::` runs only Part A's tests. Part B's tests keep failing with `todo!()` until you finish it.

Part B — `Cart` (`src/lib.rs`):

5. `LineItem::new` and `subtotal_cents`. `.to_string()` turns a `&str` into an owned `String`; `i64::from(quantity)` widens the `u32`.
6. `Cart::new` and `items`. `&self.items` is a `&Vec<LineItem>`, which coerces to `&[LineItem]`.
7. `find`: `self.items.iter().find(|item| item.sku == sku)`. The result borrows from `self`. `|item| …` is a closure, like PHP's `fn($item) => …`; lesson 06 covers closures and iterators.
8. `add`: look for an existing line with `self.items.iter_mut().find(…)`, then either bump its quantity or push `item`. Handle quantity 0 first.
9. `remove`: a private helper `fn position(&self, sku: &str) -> Option<usize>` using `iter().position(…)` is useful here and in step 10. `Vec::remove(index)` returns the owned element and keeps the order; `swap_remove` is O(1) but reorders.
10. `set_quantity`: `position`, then assign through `self.items[index]` or remove.
11. `total_cents`: a `for` loop over `&self.items`. Iterator `sum` comes in lesson 06.
12. `merge`: `for item in other.items` moves each line out of `other`; hand it to `add`.
13. `into_items`: return the field. Why doesn't this need a clone?

Optional extension: add `apply_discount(&mut self, percent: u32)` that lowers every unit price with `for item in &mut self.items`, plus your own tests. Then change the loop to `self.items.iter()` and read the error (E0594).

## Done when

- `cargo test`, `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --features broken -- -D warnings` pass.
- The Part A table is filled in.
- You can explain:
  - why `let b = a;` makes `a` unusable for a `String` but not for an `i64`
  - in which order a function's locals are dropped, and when a temporary is dropped
  - why `find` returns `Option<&LineItem>` instead of `Option<LineItem>`, and what you can't do to the cart while holding the result
  - what the caller of `merge` can no longer do with `other`, and why no `LineItem` needed cloning
  - the difference between `for item in v`, `for item in &v`, and `for item in &mut v`
