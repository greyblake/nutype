use nutype::nutype;

#[nutype(validate(min_len = 3, max_len = 255))]
pub struct Name(String);

fn main() {}
