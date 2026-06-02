use nutype::nutype;
use rust_decimal::Decimal;

// `finite` is not a valid validator for Decimal (it cannot be NaN/inf), so it
// must be reported as an unknown validation attribute.
#[nutype(
    validate(finite),
    derive(Debug),
)]
struct Amount(Decimal);

fn main() {}
