use nutype::nutype;

#[nutype(
    validate(with = validate_num, error = NumError, predicate = |val| *val > 0 )
)]
pub struct Num(i32);

fn validate_num(val: &i32) -> Result<(), NumError> {
    if *val > 100 {
        Err(NumError::TooBig)
    } else if *val < 0 {
        Err(NumError::TooSmall)
    } else {
        Ok(())
    }
}

#[derive(Debug)]
enum NumError {
    TooBig,
    TooSmall,
}

fn main () {}
