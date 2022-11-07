// TODO:
// * Address clippy warnings
// * Implement validation for number
//   * max cannot be smaller than min
//   * overlaps between clamp, min and max.
// * Support other numbers:
//   * Integers
//   * Floats
// * Support serde
//   * Serialize
//   * Deserialize
// * Support Arbitrary
// * Support decimals libraries:
//   * https://crates.io/crates/rust_decimal
// * Impl  "did you mean" hints:
//   * E.g. unknown validation rule `min`. Did you mean `min_len`?
// * Finalize syntax!
// * Setup CI

use core::convert::TryFrom;
use nutype_derive::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(present, min_len = 5)
)]
pub struct Email(String);

/// Just an age of the age.
#[nutype(
    sanitize(clamp(0, 200))
    validate(min = 0, max = 20)
)]
pub struct Value(u8);

#[nutype(validate(min_len = 5))]
pub struct Username(String);

fn main() {
    let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    println!("\n\nemail = {:?}\n\n", email);

    let value = Value::try_from(15).unwrap();
    println!("value = {value:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        #[nutype(
            sanitize(clamp(0, 200))
            validate(min = 18, max = 99)
        )]
        struct Age(u8);

        assert_eq!(Age::try_from(17), Err(AgeError::TooSmall));
        assert_eq!(Age::try_from(100), Err(AgeError::TooBig));
        assert!(Age::try_from(20).is_ok());
    }
}
