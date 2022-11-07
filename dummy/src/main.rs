// TODO:
// * Remove Vec<syn::Error> - there seem to be no use of it
// * Ensure negative numbers can be correctly parsed in:
//   * number types (validate, sanitize)
//   * string types (validate(min, max)
// * Implement validation for number
//   * max cannot be smaller than min
//   * overlaps between clamp, min and max.
// * Support other numbers:
//   * Integers
//   * Floats
// * Custom validations
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
    sanitize(clamp(0, 100))
    validate(min = 0, max = 320_100)
)]
pub struct Value(i64);

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
    fn test_u8_validate() {
        #[nutype(
            sanitize(clamp(0, 200))
            validate(min = 18, max = 99)
        )]
        struct Age(u8);

        assert_eq!(Age::try_from(17), Err(AgeError::TooSmall));
        assert_eq!(Age::try_from(100), Err(AgeError::TooBig));
        assert!(Age::try_from(20).is_ok());
    }

    #[test]
    fn test_u8_sanitize() {
        #[nutype(sanitize(clamp(10, 100)))]
        struct Percentage(u8);

        assert_eq!(Percentage::from(101), Percentage::from(100));
        assert_eq!(Percentage::from(9), Percentage::from(10));
    }

    #[test]
    fn test_u16() {
        #[nutype(validate(min = 18, max = 65000))]
        struct Age(u16);

        assert_eq!(Age::try_from(17), Err(AgeError::TooSmall));
        assert_eq!(Age::try_from(65001), Err(AgeError::TooBig));
        assert!(Age::try_from(20).is_ok());
    }

    #[test]
    fn test_u32() {
        #[nutype(validate(min = 1000, max = 100_000))]
        struct Amount(u32);

        assert_eq!(Amount::try_from(17), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(100_001), Err(AmountError::TooBig));
        assert!(Amount::try_from(100_000).is_ok());
    }

    #[test]
    fn test_u64() {
        #[nutype(validate(min = 1000, max = 18446744073709551000))]
        struct Amount(u64);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(
            Amount::try_from(18446744073709551001),
            Err(AmountError::TooBig)
        );
        assert!(Amount::try_from(1000).is_ok());
    }

    #[test]
    fn test_u128() {
        #[nutype(validate(min = 1000, max = 170141183460469231731687303715884105828))]
        struct Amount(u128);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(
            Amount::try_from(170141183460469231731687303715884105829),
            Err(AmountError::TooBig)
        );
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(170141183460469231731687303715884105828).is_ok());
    }

    #[test]
    fn test_i8_sanitize() {
        #[nutype(sanitize(clamp(0, 100)))]
        struct Percentage(i8);

        assert_eq!(Percentage::from(101), Percentage::from(100));
        assert_eq!(Percentage::from(-1), Percentage::from(0));
    }

    #[test]
    fn test_i8_validate() {
        // TODO: use negative numbers
        #[nutype(validate(min = 18, max = 99))]
        struct Age(i8);

        assert_eq!(Age::try_from(17), Err(AgeError::TooSmall));
        assert_eq!(Age::try_from(100), Err(AgeError::TooBig));
        assert!(Age::try_from(20).is_ok());
    }

    #[test]
    fn test_i16_validate() {
        #[nutype(validate(min = 1000, max = 32_000))]
        struct Amount(i16);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(32_001), Err(AmountError::TooBig));
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(32_000).is_ok());
    }

    #[test]
    fn test_i32_validate() {
        #[nutype(validate(min = 1000, max = 320_000))]
        struct Amount(i32);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(320_001), Err(AmountError::TooBig));
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(320_000).is_ok());
    }

    #[test]
    fn test_i64_validate() {
        #[nutype(validate(min = 1000, max = 8446744073709551000))]
        struct Amount(i64);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(
            Amount::try_from(8446744073709551001),
            Err(AmountError::TooBig)
        );
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(8446744073709551000).is_ok());
    }
}
