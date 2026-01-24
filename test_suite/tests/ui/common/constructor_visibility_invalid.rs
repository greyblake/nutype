use nutype::nutype;

// Test that invalid visibility value produces a clear error
#[nutype(
    sanitize(trim),
    constructor(visibility = invalid_visibility_value),
)]
pub struct Username(String);

fn main() {}
