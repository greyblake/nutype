use nutype::nutype;

// Same trait in multiple cfg_attr entries
#[nutype(
    derive(Debug),
    cfg_attr(test, derive(Clone)),
    cfg_attr(test, derive(Clone)),
)]
struct Name(String);

fn main() { }
