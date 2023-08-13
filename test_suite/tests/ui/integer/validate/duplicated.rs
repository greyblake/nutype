use nutype::nutype;

#[nutype(validate(less_or_equal = 0, less_or_equal = 0))]
pub struct Amount(i64);

fn main() {}
