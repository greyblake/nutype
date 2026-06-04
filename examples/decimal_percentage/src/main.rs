use nutype::nutype;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// A percentage in the inclusive range [0, 100].
// Note: bounds in the attribute are written as bare literals (`0`, `100`),
// while values passed at runtime are real `Decimal`s (here via `dec!`).
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display)
)]
struct Percentage(Decimal);

// A monetary amount, rounded to 2 decimal places by a sanitizer, that must be
// non-negative.
#[nutype(
    sanitize(with = |d: Decimal| d.round_dp(2)),
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq),
)]
struct Money(Decimal);

fn main() {
    // Valid percentages.
    assert_eq!(
        Percentage::try_new(dec!(42.5)).unwrap().into_inner(),
        dec!(42.5)
    );
    assert_eq!(Percentage::try_new(dec!(0)).unwrap().into_inner(), dec!(0));
    assert_eq!(
        Percentage::try_new(dec!(100)).unwrap().into_inner(),
        dec!(100)
    );

    // Out of range.
    assert_eq!(
        Percentage::try_new(dec!(150)),
        Err(PercentageError::LessOrEqualViolated),
    );
    assert_eq!(
        Percentage::try_new(dec!(-1)),
        Err(PercentageError::GreaterOrEqualViolated),
    );

    // The sanitizer rounds 9.999 -> 10.00 before validation.
    assert_eq!(
        Money::try_new(dec!(9.999)).unwrap().into_inner(),
        dec!(10.00)
    );
    assert_eq!(
        Money::try_new(dec!(-0.01)),
        Err(MoneyError::GreaterOrEqualViolated),
    );

    println!("All decimal examples passed.");
}
