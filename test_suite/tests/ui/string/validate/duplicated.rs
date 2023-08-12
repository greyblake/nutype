use nutype::nutype;

#[nutype(validate(char_len_min = 5, char_len_max = 255, char_len_min = 6))]
pub struct Email(String);

fn main () {}
