use nutype::nutype;

#[nutype(validate(len_char_min = 127, len_char_max = 63))]
pub struct Email(String);

fn main () {}
