use nutype::nutype;

// Struct-level serde attributes other than `transparent` are not supported:
// nutype generates its own serde impls, so the helper attr would not resolve.
#[nutype(derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Name(String);

fn main() {}
