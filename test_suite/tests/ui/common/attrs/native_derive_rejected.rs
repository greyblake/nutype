use nutype::nutype;

// Native #[derive(...)] must keep being rejected: the derive set is curated
// by #[nutype(derive(...))] to protect the type's invariants.
#[nutype(validate(not_empty))]
#[derive(Clone)]
pub struct Name(String);

fn main() {}
