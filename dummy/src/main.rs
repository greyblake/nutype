use nutype::nutype;

fn validate_name(name: &str) -> Result<(), NameError> {
    if name.len() < 3 {
        Err(NameError::TooShort)
    } else if name.len() > 10 {
        Err(NameError::TooLong)
    } else {
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum NameError {
    TooShort,
    TooLong,
}

#[nutype(
    validate(with = validate_name, error = NameError),
    derive(Debug, AsRef, PartialEq),
)]
struct Name(String);

fn main() {
    let name = Name::try_new("John").unwrap();
    assert_eq!(name.as_ref(), "John");

    assert_eq!(
        Name::try_new("JohnJohnJohnJohnJohn"),
        Err(NameError::TooLong)
    );

    assert_eq!(Name::try_new("Jo"), Err(NameError::TooShort));
}
