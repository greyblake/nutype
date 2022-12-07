use nutype::nutype;

#[nutype(validate(min_len = 127, max_len = 63))]
pub struct Email(String);

fn main () {}
