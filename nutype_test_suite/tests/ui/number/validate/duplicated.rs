use nutype::nutype;

#[nutype(validate(max = 0, max = 0))]
pub struct Amount(i64);

fn main() {}
