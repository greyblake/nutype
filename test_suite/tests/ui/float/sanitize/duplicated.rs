use nutype::nutype;

#[nutype(
    sanitize(
        with = |n| n,
        with = |n| n,
    )
)]
pub struct Amount(f32);

fn main() {}
