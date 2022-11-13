use nutype::nutype;

#[nutype(validate(unique))]
pub struct Email(String);

fn main () {}
