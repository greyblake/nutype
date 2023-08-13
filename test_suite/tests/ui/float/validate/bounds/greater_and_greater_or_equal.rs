use nutype::nutype;

#[nutype(validate(greater = 0.0, greater_or_equal = 0.5))]
pub struct Amount(f64);

fn main() {}
