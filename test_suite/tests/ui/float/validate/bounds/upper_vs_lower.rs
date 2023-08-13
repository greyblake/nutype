use nutype::nutype;

#[nutype(validate(less = 0, greater_or_equal = 20))]
pub struct Amount(f64);

fn main() {}
