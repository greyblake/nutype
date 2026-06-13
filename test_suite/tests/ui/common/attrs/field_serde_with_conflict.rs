use nutype::nutype;

// `with` already provides both directions; combining it with
// `serialize_with` is ambiguous and must be rejected (same as serde).
#[nutype(derive(Debug))]
pub struct Name(#[serde(with = "codec", serialize_with = "serialize_fn")] String);

fn main() {}
