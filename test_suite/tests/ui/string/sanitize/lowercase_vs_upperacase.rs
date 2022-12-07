use nutype::nutype;

#[nutype(sanitize(trim, lowercase, uppercase))]
pub struct Email(String);

fn main() {}
