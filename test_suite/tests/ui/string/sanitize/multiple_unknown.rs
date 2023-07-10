use nutype::nutype;

#[nutype(sanitize(foo, trim, bar))]
pub struct Email(String);

fn main() {}
