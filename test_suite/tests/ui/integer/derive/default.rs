use nutype::nutype;

#[nutype(
    validate(less_or_equal = 1024),
    derive(Default)
)]
pub struct Count(i32);

fn main() {}
