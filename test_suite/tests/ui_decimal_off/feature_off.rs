use nutype::nutype;

// Without the `rust_decimal` feature enabled, wrapping `Decimal` must produce a
// friendly error pointing the user at the feature flag.
#[nutype(derive(Debug))]
struct Price(rust_decimal::Decimal);

fn main() {}
