use nutype::nutype;

#[nutype(sanitize(trim))]
#[derive(Debug, Clone, Bingo, PartialEq)]
struct Name(String);

fn main() {}
