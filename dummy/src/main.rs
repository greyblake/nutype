use nutype::nutype;

#[nutype(
    validate(greater = 0, less = 24,),
    derive(Debug, PartialEq, Deref, AsRef)
)]
pub struct Hour(i32);

fn main() {}
