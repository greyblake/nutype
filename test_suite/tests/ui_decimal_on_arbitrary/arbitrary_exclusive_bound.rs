use nutype::nutype;
use rust_decimal::Decimal;

// Deriving `Arbitrary` with an exclusive bound (`greater` / `less`) is not yet
// supported for Decimal; only inclusive bounds are.
#[nutype(
    validate(greater = 0),
    derive(Debug, Arbitrary),
)]
struct Amount(Decimal);

fn main() {}
