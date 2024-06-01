use nutype::nutype;
use std::borrow::Cow;

#[nutype(
    validate(predicate = |s| s.len() >= 3),
)]
struct Clarabelle<'a>(Cow<'a, str>);

fn main() {}
