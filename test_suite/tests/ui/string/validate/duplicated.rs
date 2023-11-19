use nutype::nutype;

#[nutype(validate(len_char_min = 5, len_char_max = 255, len_char_min = 6))]
pub struct Email(String);

fn main () {}
