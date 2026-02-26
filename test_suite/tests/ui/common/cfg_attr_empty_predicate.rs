use nutype::nutype;

// Empty predicate should be rejected
#[nutype(
    derive(Debug),
    cfg_attr(, derive(Clone)),
)]
struct Name(String);

fn main() { }
