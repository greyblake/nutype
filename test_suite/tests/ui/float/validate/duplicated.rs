use nutype::nutype;

#[nutype(validate(max = 0, max = 0))]
pub struct Amount(f32);

fn main() {}
