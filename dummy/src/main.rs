use nutype::nutype;

#[nutype(validate(len_char_min = 5), default = "Anonymous", derive(Default))]
pub struct Name(String);

fn main() {}
