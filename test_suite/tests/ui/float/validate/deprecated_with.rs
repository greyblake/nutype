use nutype::nutype;

#[nutype(validate(with = |c| c>0))]
pub struct Amount(f32);

fn main() {}
