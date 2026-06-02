use nutype::nutype;
use rust_decimal::Decimal;

// `const_fn` is not supported for Decimal, because its comparison operators are
// not `const`.
#[nutype(
    const_fn,
    validate(greater_or_equal = 0),
    derive(Debug),
)]
struct Amount(Decimal);

fn main() {}
