use nutype::nutype;

#[nutype(
    derive_unchecked(::std::fmt::Debug),
)]
struct Name(String);

fn main() { }

