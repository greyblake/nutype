use nutype::nutype;

#[nutype(derive(JsonSchema))]
pub struct Username(String);

fn main() {}
