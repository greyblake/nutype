use nutype::nutype;

#[nutype(validate(maxLen = 255))]
pub struct Name(String);

fn main () {}
