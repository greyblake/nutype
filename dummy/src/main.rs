// TODO:
// * Ensure negative numbers can be correctly parsed in:
//   * number types (validate, sanitize)
//   * string types (validate(min, max)
// * Implement validation for number
//   * max cannot be smaller than min
//   * overlaps between clamp, min and max.
// * Custom validations
// * Support serde
//   * Serialize
//   * Deserialize
// * Support Arbitrary
// * Support decimals libraries:
//   * https://crates.io/crates/rust_decimal
// * Support time libraries (e.g. chrono)
// * Impl  "did you mean" hints:
//   * E.g. unknown validation rule `min`. Did you mean `min_len`?
// * Finalize syntax!
// * Setup CI
// * Find a way to bypass documentation comments
// Refactor parsers

use core::convert::TryFrom;
use nutype_derive::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(present, min_len = 5)
)]
pub struct Email(String);

/// Just an age of the age.
#[nutype(
    sanitize(clamp(-1000, -10))
    validate(min = -2000, max = -10)
)]
pub struct Value(i32);

fn main() {
    let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    println!("\n\nemail = {:?}\n\n", email);
    assert_eq!(email.into_inner(), "example@mail.org");

    let value = Value::try_from(-15).unwrap();
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

        let amount = Amount::try_from(2055).unwrap();
        assert_eq!(amount.into_inner(), 2055);
    }

    #[test]
    fn test_i32_negative() {
        #[nutype(
            sanitize(clamp(-200, -5))
            validate(min = -100, max = -50)
        )]
        pub struct Balance(i32);

        assert_eq!(Balance::try_from(-300), Err(BalanceError::TooSmall));
        assert_eq!(Balance::try_from(-4), Err(BalanceError::TooBig));

        let balance = Balance::try_from(-55).unwrap();
        assert_eq!(balance.into_inner(), -55);
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

    #[test]
    fn test_i128_validate() {
        #[nutype(validate(min = 1000, max = 70141183460469231731687303715884105000))]
        struct Amount(i128);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(
            Amount::try_from(70141183460469231731687303715884105001),
            Err(AmountError::TooBig)
        );
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(70141183460469231731687303715884105000).is_ok());
    }

    #[test]
    fn test_usize_validate() {
        #[nutype(validate(min = 1000, max = 2000))]
        struct Amount(usize);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(2001), Err(AmountError::TooBig));
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(2000).is_ok());
    }

    #[test]
    fn test_isize_validate() {
        #[nutype(validate(min = 1000, max = 2000))]
        struct Amount(isize);

        assert_eq!(Amount::try_from(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(2001), Err(AmountError::TooBig));
        assert!(Amount::try_from(1000).is_ok());
        assert!(Amount::try_from(2000).is_ok());
    }

    #[test]
    fn test_f32_validate() {
        #[nutype(validate(min = 0.0, max = 100))]
        pub struct Width(f32);

        assert_eq!(Width::try_from(-0.0001), Err(WidthError::TooSmall));
        assert_eq!(Width::try_from(100.0001), Err(WidthError::TooBig));
        assert!(Width::try_from(0.0).is_ok());
        assert!(Width::try_from(100.0).is_ok());
    }

    #[test]
    fn test_f64_validate() {
        #[nutype(validate(min = 0.0, max = 100))]
        pub struct Width(f64);

        assert_eq!(Width::try_from(-0.0001), Err(WidthError::TooSmall));
        assert_eq!(Width::try_from(100.0001), Err(WidthError::TooBig));

        assert_eq!(Width::try_from(0.0).unwrap().into_inner(), 0.0);

        let w: f64 = Width::try_from(100.0).unwrap().into();
        assert_eq!(w, 100.0);
    }

    #[test]
    fn test_f64_negative() {
        #[nutype(
            sanitize(clamp(-200.25, -5))
            validate(min = -100.25, max = -50.1)
        )]
        pub struct Balance(f64);

        assert_eq!(Balance::try_from(-300.0), Err(BalanceError::TooSmall));
        assert_eq!(Balance::try_from(-4.0), Err(BalanceError::TooBig));

        let balance = Balance::try_from(-100.24).unwrap();
        assert_eq!(balance.into_inner(), -100.24);
    }
}
