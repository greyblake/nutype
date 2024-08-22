use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate_positive_odd, error = PositivelyOddError),
    derive(Debug, FromStr),
)]
struct PositivelyOdd(i32);

#[derive(Error, Debug, PartialEq)]
enum PositivelyOddError {
    #[error("The value is negative.")]
    Negative,

    #[error("The value is even.")]
    Even,
}

fn validate_positive_odd(value: &i32) -> Result<(), PositivelyOddError> {
    if *value < 0 {
        return Err(PositivelyOddError::Negative);
    }

    if *value % 2 == 0 {
        return Err(PositivelyOddError::Even);
    }

    Ok(())
}

fn main() {
    let err = PositivelyOdd::try_new(2).unwrap_err();
    assert_eq!(err, PositivelyOddError::Even);

    let podd: PositivelyOdd = PositivelyOdd::try_new(3).unwrap();
    assert_eq!(podd.into_inner(), 3);

    let err: PositivelyOddParseError = "-3".parse::<PositivelyOdd>().unwrap_err();
    assert!(matches!(
        err,
        PositivelyOddParseError::Validate(PositivelyOddError::Negative)
    ));
}
