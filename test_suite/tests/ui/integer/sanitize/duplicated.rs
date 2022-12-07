use nutype::nutype;

#[nutype(
    sanitize(
        with = |n| n,
        with = |n| n,
    )
)]
pub struct Amount(i32);

fn main() {}
