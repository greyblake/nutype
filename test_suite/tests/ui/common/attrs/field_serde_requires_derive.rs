use nutype::nutype;

// serialize_with requires Serialize to be derived through
// #[nutype(derive(...))]; the field attr alone does nothing.
#[nutype(derive(Debug))]
pub struct Name(#[serde(serialize_with = "serialize_fn")] String);

fn main() {}
