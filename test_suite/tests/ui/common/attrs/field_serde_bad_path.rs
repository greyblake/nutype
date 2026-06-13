use nutype::nutype;

// The value must be a path, like in serde itself.
#[nutype(derive(Debug))]
pub struct Name(#[serde(serialize_with = "99 not a path")] String);

fn main() {}
