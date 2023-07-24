use nutype::nutype;

#[nutype(
    validate(finite),
    derive(PartialEq, Eq, Ord)
)]
pub struct Size(f64);

fn main() {}
