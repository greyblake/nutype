use nutype::nutype;

#[nutype(
    validate(predicate = |n| n.is_even()),
    derive(Debug, FromStr),
)]
struct Even<T: ::num::Integer>(T);

fn main() {}
