use nutype::nutype;

#[nutype(
    sanitize(with = |v| v),
    validate(predicate = |v| !v.is_empty() )
)]
struct NonEmptyVec<T>(Vec<T>);

fn main() {}
