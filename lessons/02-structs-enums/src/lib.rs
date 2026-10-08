//! Lesson 02 — model data. Replace every `todo!()` until `cargo test` passes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Eur,
    Usd,
    Jpy,
}

impl Currency {
    /// ISO code: `"EUR"`, `"USD"`, `"JPY"`.
    pub fn code(self) -> &'static str {
        todo!()
    }

    /// Digits after the decimal point: 2 for EUR and USD, 0 for JPY.
    pub fn decimals(self) -> u32 {
        todo!()
    }
}

/// An amount stored in minor units (cents for EUR), never as a float.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    minor: i64,
    currency: Currency,
}

impl Money {
    pub fn new(minor: i64, currency: Currency) -> Self {
        todo!()
    }

    /// `12` EUR → 1200 minor units; `12` JPY → 12. `None` on overflow.
    pub fn from_major(major: i64, currency: Currency) -> Option<Self> {
        todo!()
    }

    pub fn minor(&self) -> i64 {
        todo!()
    }

    pub fn currency(&self) -> Currency {
        todo!()
    }

    /// `None` if the currencies differ or the result overflows.
    pub fn checked_add(self, other: Money) -> Option<Money> {
        todo!()
    }

    /// `None` if the currencies differ or the result overflows.
    pub fn checked_sub(self, other: Money) -> Option<Money> {
        todo!()
    }

    /// `"12.50 EUR"`, `"-0.05 EUR"`, `"1250 JPY"`.
    pub fn format(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelReason {
    CustomerRequest,
    PaymentFailed,
    OutOfStock,
}

/// Allowed transitions:
/// `Pending → Paid → Shipped → Delivered`, and `Pending | Paid → Cancelled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState {
    Pending,
    Paid { amount: Money },
    Shipped { amount: Money, tracking: u64 },
    Delivered { amount: Money },
    Cancelled { reason: CancelReason },
}

impl OrderState {
    /// Only from `Pending`, and only with a positive amount.
    pub fn pay(self, amount: Money) -> Option<OrderState> {
        todo!()
    }

    /// Only from `Paid`.
    pub fn ship(self, tracking: u64) -> Option<OrderState> {
        todo!()
    }

    /// Only from `Shipped`.
    pub fn deliver(self) -> Option<OrderState> {
        todo!()
    }

    /// Only from `Pending` or `Paid`.
    pub fn cancel(self, reason: CancelReason) -> Option<OrderState> {
        todo!()
    }

    /// The paid amount, for any state that has one.
    pub fn amount(self) -> Option<Money> {
        todo!()
    }

    /// The tracking number, only while `Shipped`.
    pub fn tracking(self) -> Option<u64> {
        todo!()
    }

    /// `"pending"`, `"paid"`, `"shipped"`, `"delivered"`, `"cancelled"`.
    pub fn label(self) -> &'static str {
        todo!()
    }

    /// Refund for a `Paid` order minus `fee`.
    /// `None` if the order isn't `Paid`, the currencies differ, or the fee exceeds the amount.
    pub fn refund(self, fee: Money) -> Option<Money> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eur(minor: i64) -> Money {
        Money::new(minor, Currency::Eur)
    }

    fn shipped() -> OrderState {
        OrderState::Paid { amount: eur(1000) }.ship(42).unwrap()
    }

    #[test]
    fn currency_codes_and_decimals() {
        assert_eq!(Currency::Eur.code(), "EUR");
        assert_eq!(Currency::Usd.code(), "USD");
        assert_eq!(Currency::Jpy.code(), "JPY");
        assert_eq!(Currency::Eur.decimals(), 2);
        assert_eq!(Currency::Usd.decimals(), 2);
        assert_eq!(Currency::Jpy.decimals(), 0);
    }

    #[test]
    fn new_keeps_minor_units() {
        let money = Money::new(1250, Currency::Usd);
        assert_eq!(money.minor(), 1250);
        assert_eq!(money.currency(), Currency::Usd);
    }

    #[test]
    fn from_major_scales_by_currency_decimals() {
        assert_eq!(Money::from_major(12, Currency::Eur), Some(eur(1200)));
        assert_eq!(
            Money::from_major(12, Currency::Jpy),
            Some(Money::new(12, Currency::Jpy))
        );
        assert_eq!(Money::from_major(i64::MAX, Currency::Eur), None);
    }

    #[test]
    fn adds_same_currency() {
        assert_eq!(eur(1250).checked_add(eur(75)), Some(eur(1325)));
        assert_eq!(eur(i64::MAX).checked_add(eur(1)), None);
    }

    #[test]
    fn subtracts_same_currency() {
        assert_eq!(eur(1250).checked_sub(eur(250)), Some(eur(1000)));
        assert_eq!(eur(100).checked_sub(eur(250)), Some(eur(-150)));
        assert_eq!(eur(i64::MIN).checked_sub(eur(1)), None);
    }

    #[test]
    fn rejects_mixed_currencies() {
        let usd = Money::new(100, Currency::Usd);
        assert_eq!(eur(100).checked_add(usd), None);
        assert_eq!(eur(100).checked_sub(usd), None);
    }

    #[test]
    fn formats_two_decimal_currencies() {
        assert_eq!(eur(1250).format(), "12.50 EUR");
        assert_eq!(eur(5).format(), "0.05 EUR");
        assert_eq!(eur(0).format(), "0.00 EUR");
        assert_eq!(Money::new(100_000, Currency::Usd).format(), "1000.00 USD");
    }

    #[test]
    fn formats_negative_amounts() {
        assert_eq!(eur(-5).format(), "-0.05 EUR");
        assert_eq!(eur(-1250).format(), "-12.50 EUR");
    }

    #[test]
    fn formats_zero_decimal_currencies() {
        assert_eq!(Money::new(1250, Currency::Jpy).format(), "1250 JPY");
        assert_eq!(Money::new(-3, Currency::Jpy).format(), "-3 JPY");
    }

    #[test]
    fn happy_path_transitions() {
        let paid = OrderState::Pending.pay(eur(1000)).unwrap();
        assert_eq!(paid, OrderState::Paid { amount: eur(1000) });

        let shipped = paid.ship(42).unwrap();
        assert_eq!(
            shipped,
            OrderState::Shipped {
                amount: eur(1000),
                tracking: 42
            }
        );

        let delivered = shipped.deliver().unwrap();
        assert_eq!(delivered, OrderState::Delivered { amount: eur(1000) });
    }

    #[test]
    fn pay_requires_a_positive_amount() {
        assert_eq!(OrderState::Pending.pay(eur(0)), None);
        assert_eq!(OrderState::Pending.pay(eur(-1)), None);
    }

    #[test]
    fn rejects_invalid_transitions() {
        let paid = OrderState::Paid { amount: eur(1000) };
        let delivered = OrderState::Delivered { amount: eur(1000) };
        let cancelled = OrderState::Cancelled {
            reason: CancelReason::PaymentFailed,
        };

        assert_eq!(OrderState::Pending.ship(1), None);
        assert_eq!(OrderState::Pending.deliver(), None);
        assert_eq!(paid.pay(eur(1000)), None);
        assert_eq!(paid.deliver(), None);
        assert_eq!(shipped().ship(2), None);
        assert_eq!(delivered.deliver(), None);
        assert_eq!(cancelled.pay(eur(1000)), None);
    }

    #[test]
    fn cancels_only_before_shipping() {
        let expected = Some(OrderState::Cancelled {
            reason: CancelReason::OutOfStock,
        });
        assert_eq!(
            OrderState::Pending.cancel(CancelReason::OutOfStock),
            expected
        );
        assert_eq!(
            OrderState::Paid { amount: eur(1000) }.cancel(CancelReason::OutOfStock),
            expected
        );
        assert_eq!(shipped().cancel(CancelReason::CustomerRequest), None);
        assert_eq!(
            OrderState::Delivered { amount: eur(1000) }.cancel(CancelReason::CustomerRequest),
            None
        );
    }

    #[test]
    fn exposes_amount_and_tracking() {
        assert_eq!(OrderState::Pending.amount(), None);
        assert_eq!(
            OrderState::Paid { amount: eur(1000) }.amount(),
            Some(eur(1000))
        );
        assert_eq!(shipped().amount(), Some(eur(1000)));
        assert_eq!(shipped().tracking(), Some(42));
        assert_eq!(OrderState::Paid { amount: eur(1000) }.tracking(), None);
        assert_eq!(shipped().deliver().unwrap().tracking(), None);
    }

    #[test]
    fn labels_every_state() {
        assert_eq!(OrderState::Pending.label(), "pending");
        assert_eq!(OrderState::Paid { amount: eur(1) }.label(), "paid");
        assert_eq!(shipped().label(), "shipped");
        assert_eq!(
            OrderState::Delivered { amount: eur(1) }.label(),
            "delivered"
        );
        assert_eq!(
            OrderState::Cancelled {
                reason: CancelReason::CustomerRequest
            }
            .label(),
            "cancelled"
        );
    }

    #[test]
    fn refunds_paid_orders_minus_fee() {
        let paid = OrderState::Paid { amount: eur(1000) };
        assert_eq!(paid.refund(eur(150)), Some(eur(850)));
        assert_eq!(paid.refund(eur(1000)), Some(eur(0)));
        assert_eq!(paid.refund(eur(1001)), None);
        assert_eq!(paid.refund(Money::new(150, Currency::Usd)), None);
        assert_eq!(shipped().refund(eur(0)), None);
        assert_eq!(OrderState::Pending.refund(eur(0)), None);
    }
}
