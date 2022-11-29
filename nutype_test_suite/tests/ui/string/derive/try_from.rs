use nutype::nutype;

#[nutype(sanitize(trim))]
#[derive(TryFrom)]
struct Name(String);

fn main() {}
