use nutype::nutype;

// Default is derived conditionally but default = <value> is missing
#[nutype(
    derive(Debug),
    cfg_attr(test, derive(Default)),
)]
struct Name(String);

fn main() { }
