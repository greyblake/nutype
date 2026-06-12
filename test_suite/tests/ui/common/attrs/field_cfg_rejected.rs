use nutype::nutype;

// #[cfg] on the inner field could cfg the only field away, breaking every
// generated impl. It must be rejected explicitly.
#[nutype(derive(Debug))]
pub struct Name(#[cfg(test)] String);

fn main() {}
