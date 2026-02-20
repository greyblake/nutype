use nutype::nutype;

// cfg_attr with predicate but no attribute
#[nutype(
    derive(Debug),
    cfg_attr(test),
)]
struct Name(String);

fn main() { }
