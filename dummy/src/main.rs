use nutype::nutype;

#[nutype(validate(finite))]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Weight(f64);

fn main() {
}
