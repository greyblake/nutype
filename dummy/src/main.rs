use nutype::nutype;

#[nutype(
    sanitize(with = |v| v),
    validate(predicate = |v| v.len() > 0)
)]
struct NonEmptyVec<T>(Vec<T>);

fn main() {}
