use nutype::nutype;

#[nutype(validate(lenCharMax = 255))]
pub struct Name(String);

fn main () {}
