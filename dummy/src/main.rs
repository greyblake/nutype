use nutype::nutype;

#[nutype(validate(with = |x| x.is_empty() ))]
// #[nutype(sanitize(with = trim_name ))]
#[derive(Debug)]
pub struct Name(String);

fn main() {}
