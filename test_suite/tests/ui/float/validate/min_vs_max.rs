use nutype::nutype;

#[nutype(validate(max = 0, min = 20))]
pub struct Amount(f64);

fn main() {}
