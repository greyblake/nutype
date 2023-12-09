use nutype::nutype;

#[nutype(
    validate(predicate = |v| v),
    derive(Default),
    default = true
)]
pub struct TestData(bool);

fn main() {}
