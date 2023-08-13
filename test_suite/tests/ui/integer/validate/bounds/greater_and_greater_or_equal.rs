use nutype::nutype;

#[nutype(validate(greater = -272, greater_or_equal = -272))]
pub struct Amount(i32);

fn main() {}
