use nutype::nutype;

// #[serde(transparent)] without serde derives has nothing to apply to.
#[nutype(derive(Debug))]
#[serde(transparent)]
pub struct Name(String);

fn main() {}
