use nutype::nutype;

#[nutype(
    validate(error = NumError)
)]
pub struct Num(i32);

#[derive(Debug)]
enum NumError {
    TooBig,
    TooSmall,
}

fn main () {}
