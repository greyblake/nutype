use nutype::nutype;
use rust_decimal::Decimal;

// `Valuable` cannot be derived for Decimal, because `rust_decimal::Decimal` does
// not implement `valuable::Valuable`.
#[nutype(derive(Debug, Valuable))]
struct Amount(Decimal);

fn main() {}
