use nutype::nutype;

// Trailing tokens after derive(...) inside cfg_attr should be rejected
#[nutype(
    derive(Debug),
    cfg_attr(test, derive(Clone) some_garbage),
)]
struct Name(String);

fn main() { }
