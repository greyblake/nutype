use nutype::nutype;

// Duplicated serde keys must be rejected.
#[nutype(derive(Debug))]
pub struct Name(#[serde(serialize_with = "serialize_fn_a", serialize_with = "serialize_fn_b")] String);

fn main() {}
