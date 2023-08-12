use nutype::nutype;

#[nutype(validate(charLenMax = 255))]
pub struct Name(String);

fn main () {}
