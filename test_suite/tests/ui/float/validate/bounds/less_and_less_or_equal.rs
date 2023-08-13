use nutype::nutype;

#[nutype(validate(less = 1000.0, less_or_equal = 1000.0))]
pub struct Amount(f64);

fn main() {}
