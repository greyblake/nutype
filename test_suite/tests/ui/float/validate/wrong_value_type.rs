use nutype::nutype;

// `5i32` does not parse as `f64`. The error must point at the value
// (not falsely report `greater` as an unknown validation attribute).
#[nutype(validate(greater = 5i32))]
pub struct Ratio(f64);

fn main() {}
