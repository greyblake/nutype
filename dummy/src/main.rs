/*
use nutype::nutype;

// Validation function
fn validate_name(name: &str) -> Result<(), NameError> {
    if name.len() < 3 {
        Err(NameError::TooShort)
    } else if name.len() > 20 {
        Err(NameError::TooLong)
    } else {
        Ok(())
    }
}

// Name validation error
#[derive(Debug)]
enum NameError {
    TooShort,
    TooLong,
}

// Variant 1: with and error
#[nutype(
    sanitize(trim),
    validate(with = validate_name, error = NameError),
    derive(Debug, AsRef, PartialEq, Deref),
)]
struct Name(String);
*/

fn main() {}
