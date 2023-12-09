use nutype::nutype;

#[nutype(
    validate(predicate = |v| v),
    derive(Default),
    default = false
)]
pub struct TestData(bool);

fn main() {}
