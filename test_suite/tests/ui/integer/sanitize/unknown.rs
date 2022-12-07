use nutype::nutype;

#[nutype(sanitize(convert_to_euro))]
pub struct Amount(i32);

fn main () {}
