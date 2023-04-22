use nutype::nutype;

#[nutype(validate(finite))]
#[derive(PartialEq, PartialOrd, Ord)]
pub struct Size(f64);

fn main() {}
