use nutype::nutype;

#[nutype(derive(Debug, DieselNewType))]
pub struct DieselNewTypeString(String);

fn main() {}
