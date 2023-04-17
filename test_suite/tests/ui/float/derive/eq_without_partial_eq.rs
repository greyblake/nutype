use nutype::nutype;

#[nutype(validate(finite))]
#[derive(Eq)]
pub struct Size(f32);

fn main() {}
