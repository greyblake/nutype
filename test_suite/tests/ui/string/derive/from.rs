use nutype::nutype;

#[nutype(
    validate(not_empty),
    derive(From),
)]
struct Name(String);

fn main() {}
