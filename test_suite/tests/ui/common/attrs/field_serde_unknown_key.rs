use nutype::nutype;

// Only `with`, `serialize_with` and `deserialize_with` are supported on the
// inner field.
#[nutype(derive(Debug))]
pub struct Name(#[serde(rename = "name")] String);

fn main() {}
