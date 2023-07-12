use nutype::nutype;

#[nutype(
    validate(min = -100, max = 100)
)]
#[derive(Debug)]
pub struct Amount(i32);

fn main() {}
