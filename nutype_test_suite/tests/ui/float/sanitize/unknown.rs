use nutype::nutype;

#[nutype(sanitize(convert_to_euro))]
pub struct Amount(f64);

fn main () {}
