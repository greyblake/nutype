use nutype::nutype;

#[nutype(sanitize(), validate(min = 10, max = 2_000))]
#[derive(Debug)]
pub struct Amount(i32);

fn main() {}
