use nutype::nutype;
use rust_decimal::Decimal;

// A bare numeric literal default that is out of Decimal's range must produce a
// friendly error rather than emitting broken code.
#[nutype(
    derive(Debug, Default),
    default = 1e40,
)]
struct Amount(Decimal);

fn main() {}
