use nutype::nutype;

#[nutype(sanitize(trim))]
#[derive(Debug, , !)]
struct Name(String);

fn main() {}
