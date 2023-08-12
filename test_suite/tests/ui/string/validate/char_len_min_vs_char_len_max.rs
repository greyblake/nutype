use nutype::nutype;

#[nutype(validate(char_len_min = 127, char_len_max = 63))]
pub struct Email(String);

fn main () {}
