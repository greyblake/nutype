use nutype::nutype;

#[nutype(sanitize(foo = true, trim = true, bar = true))]
pub struct Email(String);

fn main() {}
