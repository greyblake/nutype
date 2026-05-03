use nutype::nutype;

// `0.0` is a float literal, but the inner type is `i32`.
// The error must point at the value `0.0` (not at `greater`).
#[nutype(validate(greater = 0.0))]
pub struct MaxPositionPercentage(i32);

fn main() {}
