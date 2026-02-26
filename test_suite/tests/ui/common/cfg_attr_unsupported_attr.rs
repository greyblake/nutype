use nutype::nutype;

// Only derive and derive_unchecked are supported inside cfg_attr
#[nutype(
    derive(Debug),
    cfg_attr(test, sanitize(trim)),
)]
struct Name(String);

fn main() { }
