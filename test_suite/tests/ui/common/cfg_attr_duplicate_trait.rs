use nutype::nutype;

// A trait cannot appear in both unconditional derive and cfg_attr derive
#[nutype(
    derive(Debug, Clone),
    cfg_attr(test, derive(Clone)),
)]
struct Name(String);

fn main() { }
