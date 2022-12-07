use nutype::nutype;

#[nutype(sanitize(trim, lowercase, trim))]
pub struct Email(String);

fn main() {}
