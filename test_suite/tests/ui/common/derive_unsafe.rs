use nutype::nutype;

#[nutype(
    derive_unsafe(::std::fmt::Debug),
)]
struct Name(String);

fn main() { }

