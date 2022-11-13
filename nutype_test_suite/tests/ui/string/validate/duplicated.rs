use nutype::nutype;

#[nutype(validate(min_len = 5, max_len = 255, min_len = 6))]
pub struct Email(String);

fn main () {}
