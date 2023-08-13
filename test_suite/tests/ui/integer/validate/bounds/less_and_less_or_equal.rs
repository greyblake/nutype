use nutype::nutype;

#[nutype(validate(less = 99, less_or_equal = 99))]
pub struct Amount(i16);

fn main() {}
