use nutype::nutype;

#[nutype(
    validate(greater = 0, less = 24, predicate = |x| x == &4),
    derive(Debug, PartialEq, Deref, AsRef, Default),
    default = 4
)]
pub struct Hour(i32);

fn main() {}
