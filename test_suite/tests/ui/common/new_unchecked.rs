use nutype::nutype;

#[nutype(
    new_unchecked
    sanitize(trim)
    validate(not_empty)
)]
pub struct Username(String);

fn main() {}
