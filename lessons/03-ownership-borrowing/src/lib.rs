//! Lesson 03 — ownership and borrowing.
//!
//! Part A is `src/broken.rs`, compiled only with `--features broken`.
//! Part B is the cart below: replace every `todo!()` until `cargo test` passes.

#[cfg(feature = "broken")]
pub mod broken;

/// One line of a cart. Prices are integer cents, as in lesson 02.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineItem {
    pub sku: String,
    pub name: String,
    pub unit_price_cents: i64,
    pub quantity: u32,
}

impl LineItem {
    /// Copies `sku` and `name` into owned `String`s.
    pub fn new(sku: &str, name: &str, unit_price_cents: i64, quantity: u32) -> Self {
        todo!()
    }

    /// `unit_price_cents × quantity`. Assumes the result fits in an `i64`.
    pub fn subtotal_cents(&self) -> i64 {
        todo!()
    }
}

/// A shopping cart. SKUs are compared exactly (case-sensitive).
///
/// Every method keeps these invariants:
/// - no two lines share a `sku`;
/// - every line has `quantity >= 1`;
/// - a new line is appended at the end, and no method reorders existing lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cart {
    items: Vec<LineItem>,
}

impl Cart {
    /// An empty cart.
    pub fn new() -> Self {
        todo!()
    }

    /// Every line, in cart order.
    pub fn items(&self) -> &[LineItem] {
        todo!()
    }

    /// The line with this `sku`, borrowed from the cart. `None` if there is none.
    pub fn find(&self, sku: &str) -> Option<&LineItem> {
        todo!()
    }

    /// Takes ownership of `item`.
    /// - `item.quantity == 0`: the cart is unchanged.
    /// - A line with the same `sku` exists: its quantity grows by `item.quantity`. The existing
    ///   line keeps its own `name` and `unit_price_cents`; the rest of `item` is dropped.
    /// - Otherwise `item` becomes the last line.
    ///
    /// Assumes quantities don't overflow `u32`.
    pub fn add(&mut self, item: LineItem) {
        todo!()
    }

    /// Removes the line with this `sku` and hands it back to the caller; the remaining lines keep
    /// their order. `None`, with the cart unchanged, if there is no such line.
    pub fn remove(&mut self, sku: &str) -> Option<LineItem> {
        todo!()
    }

    /// Sets the quantity of an existing line; `0` removes the line.
    /// Returns `true` if a line with this `sku` existed. Otherwise returns `false` and leaves the
    /// cart unchanged — it never adds a line.
    pub fn set_quantity(&mut self, sku: &str, quantity: u32) -> bool {
        todo!()
    }

    /// Sum of every line's subtotal; `0` for an empty cart. Assumes no overflow.
    pub fn total_cents(&self) -> i64 {
        todo!()
    }

    /// Moves every line of `other` into this cart, in `other`'s order, by the same rules as
    /// `add`. Consumes `other`; no `LineItem` is cloned.
    pub fn merge(&mut self, other: Cart) {
        todo!()
    }

    /// Consumes the cart and returns its lines in cart order.
    pub fn into_items(self) -> Vec<LineItem> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apple(quantity: u32) -> LineItem {
        LineItem::new("APL", "Apple", 50, quantity)
    }

    fn bread(quantity: u32) -> LineItem {
        LineItem::new("BRD", "Bread", 250, quantity)
    }

    fn milk(quantity: u32) -> LineItem {
        LineItem::new("MLK", "Milk", 120, quantity)
    }

    fn cart_of(items: Vec<LineItem>) -> Cart {
        let mut cart = Cart::new();
        for item in items {
            cart.add(item);
        }
        cart
    }

    fn skus(cart: &Cart) -> Vec<&str> {
        cart.items().iter().map(|item| item.sku.as_str()).collect()
    }

    #[test]
    fn line_item_new_copies_its_fields() {
        let expected = LineItem {
            sku: "APL".to_string(),
            name: "Apple".to_string(),
            unit_price_cents: 50,
            quantity: 3,
        };
        assert_eq!(LineItem::new("APL", "Apple", 50, 3), expected);
    }

    #[test]
    fn line_item_subtotal() {
        assert_eq!(apple(1).subtotal_cents(), 50);
        assert_eq!(bread(3).subtotal_cents(), 750);
        assert_eq!(LineItem::new("GFT", "Sticker", 0, 9).subtotal_cents(), 0);
    }

    #[test]
    fn new_cart_is_empty() {
        assert!(Cart::new().items().is_empty());
    }

    #[test]
    fn add_appends_new_skus_in_order() {
        let mut cart = Cart::new();
        cart.add(bread(1));
        cart.add(apple(2));
        cart.add(milk(1));
        assert_eq!(cart.items(), [bread(1), apple(2), milk(1)]);
    }

    #[test]
    fn add_merges_quantity_into_the_existing_line() {
        let mut cart = cart_of(vec![apple(2), bread(1)]);
        cart.add(LineItem::new("APL", "Green apple", 99, 3));
        assert_eq!(cart.items(), [apple(5), bread(1)]);
    }

    #[test]
    fn add_ignores_zero_quantity() {
        let mut cart = Cart::new();
        cart.add(apple(0));
        assert!(cart.items().is_empty());

        cart.add(apple(2));
        cart.add(apple(0));
        assert_eq!(cart.items(), [apple(2)]);
    }

    #[test]
    fn add_keeps_skus_case_sensitive() {
        let cart = cart_of(vec![apple(1), LineItem::new("apl", "Apple", 50, 1)]);
        assert_eq!(skus(&cart), ["APL", "apl"]);
    }

    #[test]
    fn find_borrows_a_line() {
        let cart = cart_of(vec![apple(2), bread(1)]);
        assert_eq!(cart.find("BRD"), Some(&bread(1)));
        assert_eq!(cart.find("APL").map(|item| item.quantity), Some(2));
    }

    #[test]
    fn find_returns_none_for_unknown_skus() {
        let cart = cart_of(vec![apple(2)]);
        assert_eq!(cart.find("MLK"), None);
        assert_eq!(cart.find("apl"), None);
        assert_eq!(Cart::new().find("APL"), None);
    }

    #[test]
    fn remove_hands_the_line_back_and_keeps_order() {
        let mut cart = cart_of(vec![bread(1), apple(2), milk(3)]);
        assert_eq!(cart.remove("APL"), Some(apple(2)));
        assert_eq!(cart.items(), [bread(1), milk(3)]);
        assert_eq!(cart.remove("BRD"), Some(bread(1)));
        assert_eq!(cart.items(), [milk(3)]);
    }

    #[test]
    fn remove_unknown_sku_changes_nothing() {
        let mut cart = cart_of(vec![bread(1), apple(2)]);
        assert_eq!(cart.remove("MLK"), None);
        assert_eq!(cart.items(), [bread(1), apple(2)]);
    }

    #[test]
    fn set_quantity_updates_an_existing_line() {
        let mut cart = cart_of(vec![bread(1), apple(2)]);
        assert!(cart.set_quantity("APL", 7));
        assert_eq!(cart.items(), [bread(1), apple(7)]);
    }

    #[test]
    fn set_quantity_zero_removes_the_line() {
        let mut cart = cart_of(vec![bread(1), apple(2), milk(3)]);
        assert!(cart.set_quantity("APL", 0));
        assert_eq!(cart.items(), [bread(1), milk(3)]);
    }

    #[test]
    fn set_quantity_on_unknown_sku_changes_nothing() {
        let mut cart = cart_of(vec![bread(1)]);
        assert!(!cart.set_quantity("APL", 3));
        assert!(!cart.set_quantity("APL", 0));
        assert_eq!(cart.items(), [bread(1)]);
    }

    #[test]
    fn total_sums_every_subtotal() {
        assert_eq!(Cart::new().total_cents(), 0);

        let mut cart = cart_of(vec![apple(3), bread(2)]);
        assert_eq!(cart.total_cents(), 650);
        cart.add(milk(1));
        cart.set_quantity("BRD", 1);
        assert_eq!(cart.total_cents(), 520);
    }

    #[test]
    fn merge_moves_lines_and_combines_skus() {
        let mut cart = cart_of(vec![apple(1), bread(1)]);
        let other = cart_of(vec![milk(2), LineItem::new("APL", "Green apple", 99, 3)]);
        cart.merge(other);
        assert_eq!(cart.items(), [apple(4), bread(1), milk(2)]);
    }

    #[test]
    fn merge_into_an_empty_cart_keeps_the_other_order() {
        let mut cart = Cart::new();
        cart.merge(cart_of(vec![milk(1), apple(2)]));
        assert_eq!(cart.items(), [milk(1), apple(2)]);
    }

    #[test]
    fn merge_with_an_empty_cart_changes_nothing() {
        let mut cart = cart_of(vec![bread(1)]);
        cart.merge(Cart::new());
        assert_eq!(cart.items(), [bread(1)]);
    }

    #[test]
    fn into_items_returns_the_lines_in_order() {
        let cart = cart_of(vec![bread(1), apple(2)]);
        assert_eq!(cart.into_items(), vec![bread(1), apple(2)]);
        assert!(Cart::new().into_items().is_empty());
    }

    #[test]
    fn clone_is_an_independent_deep_copy() {
        let mut cart = cart_of(vec![apple(1)]);
        let snapshot = cart.clone();
        cart.add(apple(1));
        cart.add(bread(1));
        assert_eq!(snapshot.items(), [apple(1)]);
        assert_eq!(cart.items(), [apple(2), bread(1)]);
    }
}
