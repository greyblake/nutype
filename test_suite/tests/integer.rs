use nutype::nutype;

#[cfg(test)]
mod sanitizers {
    use super::*;

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            /// Some documentation for Cent goes here.
            #[nutype(sanitize(with = |n: i32| n.clamp(0, 100)))]
            pub struct Cent(i32);

            assert_eq!(Cent::new(-10).into_inner(), 0);
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(sanitize(with = |n| n.clamp(0, 100)))]
            pub struct Cent(i32);

            assert_eq!(Cent::new(-10).into_inner(), 0);
        }

        fn sanitize_cent(value: i32) -> i32 {
            value.clamp(0, 100)
        }

        #[test]
        fn test_with_function() {
            #[nutype(sanitize(with = sanitize_cent))]
            pub struct Cent(i32);

            assert_eq!(Cent::new(222).into_inner(), 100);
        }
    }

    #[test]
    fn test_from_trait() {
        #[nutype(
            sanitize(with = |a| a.clamp(18, 99)),
            derive(From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash)
        )]
        struct Age(u8);

        assert_eq!(Age::from(17).into_inner(), 18);
    }
}

#[cfg(test)]
mod validators {
    use super::*;

    #[test]
    fn test_min() {
        #[nutype(
            validate(min = 18),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u8);

        assert_eq!(Age::new(17).unwrap_err(), AgeError::TooSmall);
        assert_eq!(Age::new(18).unwrap().into_inner(), 18);
    }

    #[test]
    fn test_max() {
        #[nutype(
            validate(max = 99),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u8);

        assert_eq!(Age::new(100).unwrap_err(), AgeError::TooBig);
        assert_eq!(Age::new(99).unwrap().into_inner(), 99);
    }

    #[test]
    fn test_min_and_max() {
        #[nutype(
            validate(min = 18, max = 99),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u8);

        assert_eq!(Age::new(17).unwrap_err(), AgeError::TooSmall);
        assert_eq!(Age::new(100).unwrap_err(), AgeError::TooBig);
        assert_eq!(Age::new(25).unwrap().into_inner(), 25);
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(validate(predicate = |c: &i32| (0..=100).contains(c) ), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
            pub struct Cent(i32);

            assert_eq!(Cent::new(-10), Err(CentError::Invalid));
            assert_eq!(Cent::new(101), Err(CentError::Invalid));
            assert_eq!(Cent::new(100).unwrap().into_inner(), 100);
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(validate(predicate = |c| (0..=100).contains(c) ), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
            pub struct Cent(i32);

            assert_eq!(Cent::new(-10), Err(CentError::Invalid));
            assert_eq!(Cent::new(101), Err(CentError::Invalid));
            assert_eq!(Cent::new(100).unwrap().into_inner(), 100);
        }

        fn is_cent_valid(val: &i32) -> bool {
            (0..=100).contains(val)
        }

        #[test]
        fn test_with_function() {
            #[nutype(
                validate(predicate = is_cent_valid),
                derive(TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash),
            )]
            pub struct Cent(i32);

            assert_eq!(Cent::new(-1), Err(CentError::Invalid));
            assert_eq!(Cent::new(101), Err(CentError::Invalid));
            assert_eq!(Cent::new(100).unwrap().into_inner(), 100);
        }
    }

    #[test]
    fn test_try_from_trait() {
        #[nutype(
            validate(min = 18),
            derive(
                TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u8);

        assert_eq!(Age::try_from(17).unwrap_err(), AgeError::TooSmall);
        assert_eq!(Age::try_from(18).unwrap().into_inner(), 18);
    }

    #[test]
    fn test_try_from_trait_without_validation() {
        #[nutype(derive(Debug, PartialEq, TryFrom))]
        struct Age(u8);

        assert_eq!(Age::try_from(78).unwrap().into_inner(), 78);
    }

    #[cfg(test)]
    mod error {
        use super::*;

        #[test]
        fn test_error_display() {
            #[nutype(
                validate(min = 18),
                derive(
                    TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef,
                    Hash
                )
            )]
            struct Age(u8);

            let err = Age::try_from(17).unwrap_err();

            assert_eq!(err.to_string(), "too small");
        }
    }
}

#[cfg(test)]
mod types {
    use super::*;

    #[test]
    fn test_u8_validate() {
        #[nutype(
            sanitize(with = |n| n.clamp(0, 200)),
            validate(min = 18, max = 99),
            derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash)
        )]
        struct Age(u8);

        assert_eq!(Age::new(17), Err(AgeError::TooSmall));
        assert_eq!(Age::new(100), Err(AgeError::TooBig));
        assert!(Age::new(20).is_ok());
    }

    #[test]
    fn test_u8_sanitize() {
        #[nutype(sanitize(with = |n| n.clamp(10, 100)), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
        struct Percentage(u8);

        assert_eq!(Percentage::new(101), Percentage::new(100));
        assert_eq!(Percentage::new(9), Percentage::new(10));
    }

    #[test]
    fn test_u16() {
        #[nutype(
            validate(min = 18, max = 65000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u16);

        assert_eq!(Age::new(17), Err(AgeError::TooSmall));
        assert_eq!(Age::new(65001), Err(AgeError::TooBig));
        assert!(Age::new(20).is_ok());
    }

    #[test]
    fn test_u32() {
        #[nutype(
            validate(min = 1000, max = 100_000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(u32);

        assert_eq!(Amount::new(17), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(100_001), Err(AmountError::TooBig));
        assert!(Amount::new(100_000).is_ok());
    }

    #[test]
    fn test_u64() {
        #[nutype(
            validate(min = 1000, max = 18446744073709551000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(u64);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(18446744073709551001), Err(AmountError::TooBig));
        assert!(Amount::new(1000).is_ok());
    }

    #[test]
    fn test_u128() {
        #[nutype(
            validate(min = 1000, max = 170141183460469231731687303715884105828),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(u128);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(
            Amount::new(170141183460469231731687303715884105829),
            Err(AmountError::TooBig)
        );
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(170141183460469231731687303715884105828).is_ok());
    }

    #[test]
    fn test_i8_sanitize() {
        #[nutype(sanitize(with = |n| n.clamp(0, 100)), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
        struct Percentage(i8);

        assert_eq!(Percentage::new(101), Percentage::new(100));
        assert_eq!(Percentage::new(-1), Percentage::new(0));
    }

    #[test]
    fn test_i8_validate() {
        #[nutype(validate(min = -20, max = 100), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
        struct Offset(i8);

        assert_eq!(Offset::new(-21), Err(OffsetError::TooSmall));
        assert_eq!(Offset::new(101), Err(OffsetError::TooBig));
        assert!(Offset::new(100).is_ok());
        assert!(Offset::new(-20).is_ok());
        assert!(Offset::new(0).is_ok());
    }

    #[test]
    fn test_i16_validate() {
        #[nutype(
            validate(min = 1000, max = 32_000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(i16);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(32_001), Err(AmountError::TooBig));
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(32_000).is_ok());
    }

    #[test]
    fn test_i32_validate() {
        #[nutype(
            validate(min = 1000, max = 320_000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]

        struct Amount(i32);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(320_001), Err(AmountError::TooBig));
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(320_000).is_ok());

        let amount = Amount::new(2055).unwrap();
        assert_eq!(amount.into_inner(), 2055);
    }

    #[test]
    fn test_i32_negative() {
        #[nutype(
            sanitize(with = |n| n.clamp(-200, -5)),
            validate(min = -100, max = -50),
            derive(TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash),
        )]
        pub struct Balance(i32);

        assert_eq!(Balance::new(-300), Err(BalanceError::TooSmall));
        assert_eq!(Balance::new(-4), Err(BalanceError::TooBig));

        let balance = Balance::new(-55).unwrap();
        assert_eq!(balance.into_inner(), -55);
    }

    #[test]
    fn test_i64_validate() {
        #[nutype(
            validate(min = 1000, max = 8446744073709551000),
            derive(Debug, PartialEq)
        )]
        struct Amount(i64);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(8446744073709551001), Err(AmountError::TooBig));
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(8446744073709551000).is_ok());
    }

    #[test]
    fn test_i128_validate() {
        #[nutype(
            validate(min = 1000, max = 70141183460469231731687303715884105000),
            derive(Debug, PartialEq)
        )]
        struct Amount(i128);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(
            Amount::new(70141183460469231731687303715884105001),
            Err(AmountError::TooBig)
        );
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(70141183460469231731687303715884105000).is_ok());
    }

    #[test]
    fn test_usize_validate() {
        #[nutype(validate(min = 1000, max = 2000), derive(Debug, PartialEq))]
        struct Amount(usize);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(2001), Err(AmountError::TooBig));
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(2000).is_ok());
    }

    #[test]
    fn test_isize_validate() {
        #[nutype(validate(min = 1000, max = 2000), derive(Debug, PartialEq))]
        struct Amount(isize);

        assert_eq!(Amount::new(999), Err(AmountError::TooSmall));
        assert_eq!(Amount::new(2001), Err(AmountError::TooBig));
        assert!(Amount::new(1000).is_ok());
        assert!(Amount::new(2000).is_ok());
    }
}

#[cfg(test)]
mod visibility {
    mod encapsulated {
        use nutype::nutype;

        #[nutype(sanitize(with = |n: i32| n.clamp(0, 100)))]
        pub struct Percentage(i32);
    }

    #[test]
    fn test_public_visibility() {
        let percentage = encapsulated::Percentage::new(133);
        assert_eq!(percentage.into_inner(), 100);
    }
}

#[cfg(test)]
mod traits {
    use super::*;
    use test_suite::test_helpers::traits::*;

    #[test]
    fn test_without_validation() {
        #[nutype(derive(Debug, From, FromStr, Borrow, Clone, Copy))]
        pub struct Number(i8);

        should_implement_debug::<Number>();
        should_implement_from::<Number, i8>();
        should_implement_from_str::<Number>();
        should_implement_borrow::<Number, i8>();
        should_implement_clone::<Number>();
        should_implement_copy::<Number>();
    }

    #[test]
    fn test_with_validaiton() {
        #[nutype(
            validate(max = 1000),
            derive(Debug, TryFrom, FromStr, Borrow, Clone, Copy)
        )]
        pub struct Number(u128);

        should_implement_debug::<Number>();
        should_implement_try_from::<Number, u128>();
        should_implement_from_str::<Number>();
        should_implement_borrow::<Number, u128>();
        should_implement_clone::<Number>();
        should_implement_copy::<Number>();
    }

    #[test]
    fn test_trait_into() {
        #[nutype(derive(Into))]
        pub struct Age(u8);

        let age = Age::new(32);
        let age: u8 = age.into();
        assert_eq!(age, 32);
    }

    #[test]
    fn test_trait_from() {
        #[nutype(derive(From))]
        pub struct Amount(u32);

        let amount = Amount::from(350);
        assert_eq!(amount.into_inner(), 350);
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype(derive(AsRef))]
        pub struct Age(u8);

        let age = Age::new(32);
        let age_ref: &u8 = age.as_ref();
        assert_eq!(age_ref, &32);
    }

    #[test]
    fn test_trait_deref() {
        #[nutype(derive(Deref))]
        pub struct Number(i16);

        let magic = Number::new(42);
        assert_eq!(*magic, 42);
    }

    #[test]
    fn test_trait_borrow() {
        use std::borrow::Borrow;

        #[nutype(derive(Borrow))]
        pub struct Age(u8);

        let age = Age::new(32);
        let age_borrowed: &u8 = age.borrow();
        assert_eq!(age_borrowed, &32);
    }

    #[test]
    fn test_trait_try_from() {
        #[nutype(validate(max = 1000), derive(Debug, TryFrom))]
        pub struct Amount(i64);

        let amount = Amount::try_from(1000).unwrap();
        assert_eq!(amount.into_inner(), 1000);

        let error = Amount::try_from(1001).unwrap_err();
        assert_eq!(error, AmountError::TooBig);
    }

    #[test]
    fn test_trait_from_str_without_validation() {
        #[nutype(derive(Debug, FromStr))]
        pub struct Age(i16);

        let age: Age = "33".parse().unwrap();
        assert_eq!(age.into_inner(), 33);

        let err: AgeParseError = "foobar".parse::<Age>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Age: invalid digit found in string"
        );
    }

    #[test]
    fn test_trait_from_str_with_validation() {
        #[nutype(validate(max = 99), derive(Debug, FromStr))]
        pub struct Age(isize);

        // Happy path
        let age: Age = "99".parse().unwrap();
        assert_eq!(age.into_inner(), 99);

        let err: AgeParseError = "foobar".parse::<Age>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Age: invalid digit found in string"
        );

        // Unhappy path: validation error
        let err: AgeParseError = "101".parse::<Age>().unwrap_err();
        assert_eq!(err.to_string(), "Failed to parse Age: too big");
    }

    #[test]
    fn test_trait_display() {
        #[nutype(derive(Display))]
        pub struct Age(i64);

        let age = Age::new(35);
        assert_eq!(age.to_string(), "35");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_trait_serialize() {
        #[nutype(derive(Serialize))]
        pub struct Offset(i64);

        let offset = Offset::new(-280);
        let offset_json = serde_json::to_string(&offset).unwrap();
        assert_eq!(offset_json, "-280");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_trait_deserialize_without_validation() {
        #[nutype(derive(Deserialize))]
        pub struct Offset(i64);

        {
            let res: Result<Offset, _> = serde_json::from_str("three");
            assert!(res.is_err());
        }

        {
            let offset: Offset = serde_json::from_str("-259").unwrap();
            assert_eq!(offset.into_inner(), -259);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_trait_deserialize_with_validation() {
        #[nutype(validate(min = 13), derive(Deserialize))]
        pub struct Offset(i64);

        {
            let res: Result<Offset, _> = serde_json::from_str("three");
            assert!(res.is_err());
        }

        {
            let res: Result<Offset, _> = serde_json::from_str("12");
            assert!(res.is_err());
        }

        {
            let offset: Offset = serde_json::from_str("13").unwrap();
            assert_eq!(offset.into_inner(), 13);
        }
    }

    #[cfg(test)]
    mod trait_default {
        use super::*;

        #[test]
        fn test_default_without_validation() {
            #[nutype(default = 13, derive(Default))]
            pub struct Number(i8);

            assert_eq!(Number::default().into_inner(), 13);
        }

        #[test]
        fn test_default_with_validation_when_valid() {
            #[nutype(validate(max = 20), default = 13, derive(Default))]
            pub struct Number(i8);

            assert_eq!(Number::default().into_inner(), 13);
        }

        #[test]
        #[should_panic(expected = "Default value for type Number is invalid")]
        fn test_default_with_validation_when_invalid() {
            #[nutype(validate(max = 20), default = 21, derive(Default))]
            pub struct Number(i16);

            Number::default();
        }
    }
}

#[cfg(test)]
#[cfg(feature = "new_unchecked")]
mod new_unchecked {
    use super::*;

    #[test]
    fn test_new_unchecked() {
        #[nutype(new_unchecked, validate(min = 50))]
        pub struct Dist(u32);

        let dist = unsafe { Dist::new_unchecked(3) };
        assert_eq!(dist.into_inner(), 3);
    }
}

#[cfg(test)]
#[cfg(feature = "schemars08")]
mod derive_schemars_json_schema {
    use super::*;
    use schemars::{schema_for, JsonSchema};

    #[test]
    fn test_json_schema_derive() {
        #[nutype(derive(JsonSchema))]
        pub struct CustomerId(i128);

        assert_eq!(CustomerId::schema_name(), "CustomerId");
        // Make sure it compiles
        let _schema = schema_for!(CustomerId);
    }
}
