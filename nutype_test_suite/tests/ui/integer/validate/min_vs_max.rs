use nutype::nutype;

#[nutype(validate(max = 0, min = 20))]
pub struct Amount(i64);

fn main() {}
