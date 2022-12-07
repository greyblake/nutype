use nutype::nutype;

#[nutype(sanitize(cleanup = true))]
pub struct Email(String);

fn main() {}
