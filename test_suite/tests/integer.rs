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
    fn test_greater() {
        #[nutype(validate(greater = 18), derive(Debug))]
        struct Age(u8);

        assert_eq!(Age::try_new(18).unwrap_err(), AgeError::GreaterViolated);
        assert_eq!(Age::try_new(19).unwrap().into_inner(), 19);
    }

    #[test]
    fn test_greater_or_equal() {
        #[nutype(validate(greater_or_equal = 18), derive(Debug))]
        struct Age(u8);

        assert_eq!(
            Age::try_new(17).unwrap_err(),
            AgeError::GreaterOrEqualViolated
        );
        assert_eq!(Age::try_new(18).unwrap().into_inner(), 18);
    }

    #[test]
    fn test_less() {
        #[nutype(validate(less = -33), derive(Debug))]
        struct Degree(i32);

        assert_eq!(Degree::try_new(-33).unwrap_err(), DegreeError::LessViolated);
        assert_eq!(Degree::try_new(-34).unwrap().into_inner(), -34);
    }

    #[test]
    fn test_less_or_equal() {
        #[nutype(validate(less_or_equal = 99), derive(Debug))]
        struct Age(u8);

        assert_eq!(
            Age::try_new(100).unwrap_err(),
            AgeError::LessOrEqualViolated
        );
        assert_eq!(Age::try_new(99).unwrap().into_inner(), 99);
    }

    #[test]
    fn test_greater_or_equal_and_less_or_equal() {
        #[nutype(validate(greater_or_equal = 18, less_or_equal = 99), derive(Debug))]
        struct Age(u8);

        assert_eq!(
            Age::try_new(17).unwrap_err(),
            AgeError::GreaterOrEqualViolated
        );
        assert_eq!(
            Age::try_new(100).unwrap_err(),
            AgeError::LessOrEqualViolated
        );
        assert_eq!(Age::try_new(25).unwrap().into_inner(), 25);
    }

    mod when_boundaries_defined_as_constants {
        use super::*;

        const MIN_MINUTE: i32 = 0;
        const MAX_MINUTE: i32 = 59;
        const MIN_HOUR: i32 = 0;
        const MAX_HOUR: i32 = 25; // this is weird, but it's just for the sake of testing

        // Inclusive range
        #[nutype(validate(greater_or_equal = MIN_MINUTE, less_or_equal = MAX_MINUTE), derive(Debug))]
        struct Minute(i32);

        // Exclusive range
        #[nutype(validate(greater = MIN_HOUR, less = MAX_HOUR), derive(Debug))]
        struct Hour(i32);

        #[test]
        fn test_boundaries_defined_as_constants() {
            assert_eq!(
                Minute::try_new(-1).unwrap_err(),
                MinuteError::GreaterOrEqualViolated
            );
            assert_eq!(Minute::try_new(0).unwrap().into_inner(), 0);
            assert_eq!(
                Minute::try_new(60).unwrap_err(),
                MinuteError::LessOrEqualViolated
            );
            assert_eq!(Minute::try_new(59).unwrap().into_inner(), 59);

            assert_eq!(Hour::try_new(0).unwrap_err(), HourError::GreaterViolated);
            assert_eq!(Hour::try_new(1).unwrap().into_inner(), 1);
            assert_eq!(Hour::try_new(25).unwrap_err(), HourError::LessViolated);
            assert_eq!(Hour::try_new(24).unwrap().into_inner(), 24);
        }
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(validate(predicate = |c: &i32| (0..=100).contains(c) ), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
            pub struct Cent(i32);

            assert_eq!(Cent::try_new(-10), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(101), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100).unwrap().into_inner(), 100);
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(validate(predicate = |c| (0..=100).contains(c) ), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
            pub struct Cent(i32);

            assert_eq!(Cent::try_new(-10), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(101), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100).unwrap().into_inner(), 100);
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

            assert_eq!(Cent::try_new(-1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(101), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100).unwrap().into_inner(), 100);
        }
    }

    #[test]
    fn test_try_from_trait() {
        #[nutype(
            validate(greater_or_equal = 18),
            derive(
                TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u8);

        assert_eq!(
            Age::try_from(17).unwrap_err(),
            AgeError::GreaterOrEqualViolated
        );
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
                validate(greater_or_equal = 18),
                derive(
                    TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef,
                    Hash
                )
            )]
            struct Age(u8);

            let err = Age::try_from(17).unwrap_err();

            assert_eq!(
                err.to_string(),
                "Age is too small. The value must be greater or equal to 18."
            );
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
            validate(greater_or_equal = 18, less_or_equal = 99),
            derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash)
        )]
        struct Age(u8);

        assert_eq!(Age::try_new(17), Err(AgeError::GreaterOrEqualViolated));
        assert_eq!(Age::try_new(100), Err(AgeError::LessOrEqualViolated));
        assert!(Age::try_new(20).is_ok());
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
            validate(greater_or_equal = 18, less_or_equal = 65000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Age(u16);

        assert_eq!(Age::try_new(17), Err(AgeError::GreaterOrEqualViolated));
        assert_eq!(Age::try_new(65001), Err(AgeError::LessOrEqualViolated));
        assert!(Age::try_new(20).is_ok());
    }

    #[test]
    fn test_u32() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 100_000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(u32);

        assert_eq!(
            Amount::try_new(17),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(100_001),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(100_000).is_ok());
    }

    #[test]
    fn test_u64() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 18446744073709551000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(u64);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(18446744073709551001),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(1000).is_ok());
    }

    #[test]
    fn test_u128() {
        #[nutype(
            validate(
                greater_or_equal = 1000,
                less_or_equal = 170141183460469231731687303715884105828
            ),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(u128);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(170141183460469231731687303715884105829),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(170141183460469231731687303715884105828).is_ok());
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
        #[nutype(validate(greater_or_equal = -20, less_or_equal = 100), derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash))]
        struct Offset(i8);

        assert_eq!(
            Offset::try_new(-21),
            Err(OffsetError::GreaterOrEqualViolated)
        );
        assert_eq!(Offset::try_new(101), Err(OffsetError::LessOrEqualViolated));
        assert!(Offset::try_new(100).is_ok());
        assert!(Offset::try_new(-20).is_ok());
        assert!(Offset::try_new(0).is_ok());
    }

    #[test]
    fn test_i16_validate() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 32_000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        struct Amount(i16);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(32_001),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(32_000).is_ok());
    }

    #[test]
    fn test_i32_validate() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 320_000),
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]

        struct Amount(i32);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(320_001),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(320_000).is_ok());

        let amount = Amount::try_new(2055).unwrap();
        assert_eq!(amount.into_inner(), 2055);
    }

    #[test]
    fn test_i32_negative() {
        #[nutype(
            sanitize(with = |n| n.clamp(-200, -5)),
            validate(greater_or_equal = -100, less_or_equal = -50),
            derive(TryFrom, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash),
        )]
        pub struct Balance(i32);

        assert_eq!(
            Balance::try_new(-300),
            Err(BalanceError::GreaterOrEqualViolated)
        );
        assert_eq!(Balance::try_new(-4), Err(BalanceError::LessOrEqualViolated));

        let balance = Balance::try_new(-55).unwrap();
        assert_eq!(balance.into_inner(), -55);
    }

    #[test]
    fn test_i64_validate() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 8446744073709551000),
            derive(Debug, PartialEq)
        )]
        struct Amount(i64);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(8446744073709551001),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(8446744073709551000).is_ok());
    }

    #[test]
    fn test_i128_validate() {
        #[nutype(
            validate(
                greater_or_equal = 1000,
                less_or_equal = 70141183460469231731687303715884105000
            ),
            derive(Debug, PartialEq)
        )]
        struct Amount(i128);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_new(70141183460469231731687303715884105001),
            Err(AmountError::LessOrEqualViolated)
        );
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(70141183460469231731687303715884105000).is_ok());
    }

    #[test]
    fn test_usize_validate() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 2000),
            derive(Debug, PartialEq)
        )]
        struct Amount(usize);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(Amount::try_new(2001), Err(AmountError::LessOrEqualViolated));
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(2000).is_ok());
    }

    #[test]
    fn test_isize_validate() {
        #[nutype(
            validate(greater_or_equal = 1000, less_or_equal = 2000),
            derive(Debug, PartialEq)
        )]
        struct Amount(isize);

        assert_eq!(
            Amount::try_new(999),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(Amount::try_new(2001), Err(AmountError::LessOrEqualViolated));
        assert!(Amount::try_new(1000).is_ok());
        assert!(Amount::try_new(2000).is_ok());
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
    fn test_with_validation() {
        #[nutype(
            validate(less_or_equal = 1000),
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
        use core::borrow::Borrow;

        #[nutype(derive(Borrow))]
        pub struct Age(u8);

        let age = Age::new(32);
        let age_borrowed: &u8 = age.borrow();
        assert_eq!(age_borrowed, &32);
    }

    #[test]
    fn test_trait_try_from() {
        #[nutype(validate(less_or_equal = 1000), derive(Debug, TryFrom))]
        pub struct Amount(i64);

        let amount = Amount::try_from(1000).unwrap();
        assert_eq!(amount.into_inner(), 1000);

        let error = Amount::try_from(1001).unwrap_err();
        assert_eq!(error, AmountError::LessOrEqualViolated);
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
            "Failed to parse Age: ParseIntError { kind: InvalidDigit }"
        );
    }

    #[test]
    fn test_trait_from_str_with_validation() {
        #[nutype(validate(less_or_equal = 99), derive(Debug, FromStr))]
        pub struct Age(isize);

        // Happy path
        let age: Age = "99".parse().unwrap();
        assert_eq!(age.into_inner(), 99);

        let err: AgeParseError = "foobar".parse::<Age>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Age: ParseIntError { kind: InvalidDigit }"
        );

        // Unhappy path: validation error
        let err: AgeParseError = "101".parse::<Age>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Age: Age is too big. The value must be less or equal to 99."
        );
    }

    #[test]
    fn test_trait_display() {
        #[nutype(derive(Display))]
        pub struct Age(i64);

        let age = Age::new(35);
        assert_eq!(age.to_string(), "35");
    }

    #[cfg(feature = "serde")]
    mod serialization {
        use super::*;

        mod json_format {
            use super::*;

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
                #[nutype(validate(greater_or_equal = 13), derive(Deserialize))]
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
        }

        mod ron_format {
            use super::*;

            #[test]
            fn test_ron_roundtrip() {
                #[nutype(derive(Serialize, Deserialize, PartialEq, Debug))]
                pub struct Weight(i32);

                let weight = Weight::new(33);

                let serialized = ron::to_string(&weight).unwrap();
                let deserialized: Weight = ron::from_str(&serialized).unwrap();

                assert_eq!(deserialized, weight);
            }
        }

        mod message_pack_format {
            use super::*;

            #[test]
            fn test_rmp_roundtrip() {
                #[nutype(derive(Serialize, Deserialize, PartialEq, Debug))]
                pub struct Weight(u8);

                let weight = Weight::new(102);

                let bytes = rmp_serde::to_vec(&weight).unwrap();
                let deserialized: Weight = rmp_serde::from_slice(&bytes).unwrap();

                assert_eq!(deserialized, weight);
            }
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
            #[nutype(validate(less_or_equal = 20), default = 13, derive(Default))]
            pub struct Number(i8);

            assert_eq!(Number::default().into_inner(), 13);
        }

        #[test]
        #[should_panic(expected = "Default value for type `Number` is invalid")]
        fn test_default_with_validation_when_invalid() {
            #[nutype(validate(less_or_equal = 20), default = 21, derive(Default))]
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
        #[nutype(new_unchecked, validate(greater_or_equal = 50))]
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

#[cfg(test)]
#[cfg(feature = "valuable")]
mod derive_valuable {
    use super::*;
    use valuable::Valuable;

    #[test]
    fn test_valuable_derive() {
        #[nutype(derive(Valuable))]
        pub struct Age(u32);

        assert_eq!(format!("{:?}", Age::new(25).as_value()), r#"Age(25)"#);
    }
}

mod custom_error {
    use super::*;
    use thiserror::Error;

    #[nutype(
        validate(with = validate_positively_odd, error = PositivelyOddError),
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

    fn validate_positively_odd(value: &i32) -> Result<(), PositivelyOddError> {
        if *value < 0 {
            return Err(PositivelyOddError::Negative);
        }

        if *value % 2 == 0 {
            return Err(PositivelyOddError::Even);
        }

        Ok(())
    }

    #[test]
    fn test_custom_error() {
        {
            let err = PositivelyOdd::try_new(-1).unwrap_err();
            assert_eq!(err, PositivelyOddError::Negative);
        }

        {
            let err = PositivelyOdd::try_new(2).unwrap_err();
            assert_eq!(err, PositivelyOddError::Even);
        }

        let podd: PositivelyOdd = PositivelyOdd::try_new(3).unwrap();
        assert_eq!(podd.into_inner(), 3);
    }
}

mod constants {
    use super::*;

    const fn clamp_age(value: u8) -> u8 {
        if value > 100 {
            return 100;
        } else {
            return value;
        }
    }

    #[test]
    fn test_const_fn() {
        #[nutype(const_fn)]
        pub struct Age(u8);

        const ADULT_AGE: Age = Age::new(18);

        const DOUBLE_AGE: u8 = ADULT_AGE.into_inner() * 2;

        assert_eq!(ADULT_AGE.into_inner(), 18);
        assert_eq!(DOUBLE_AGE, 36);
    }

    #[test]
    fn test_const_fn_with_sanitize() {
        #[nutype(
            const_fn,
            sanitize(with = clamp_age),
        )]
        pub struct Age(u8);

        const BIG_AGE: Age = Age::new(150);

        assert_eq!(BIG_AGE.into_inner(), 100);
    }

    #[test]
    fn test_const_fn_with_sanitize_and_validate() {
        #[nutype(
            const_fn,
            sanitize(with = clamp_age),
            validate(greater_or_equal = 18),
        )]
        pub struct Age(u8);

        const fn unwrap(result: Result<Age, AgeError>) -> Age {
            match result {
                Ok(value) => value,
                Err(_) => panic!("const unwrap() failed"),
            }
        }

        const MID_AGE: Age = unwrap(Age::try_new(35));

        assert_eq!(MID_AGE.into_inner(), 35);
    }

    mod const_into_inner_with_copy {
        use super::*;

        // This test demonstrates that moving semantic in `const` functions works differently than
        // usually. Despite Meter does not implement `Copy` trait, it's possible to "move" it and
        // then use it again.

        #[nutype(const_fn)]
        pub struct Meter(i32);

        #[nutype(const_fn)]
        pub struct Distance(Meter);

        const METERS: Meter = Meter::new(100);
        const DISTANCE: Distance = Distance::new(METERS);

        const SAME_METERS: Meter = DISTANCE.into_inner();

        #[test]
        fn test_const_into_inner_without_copy() {
            assert_eq!(METERS.into_inner(), 100);
            assert_eq!(SAME_METERS.into_inner(), 100);
            assert_eq!(DISTANCE.into_inner().into_inner(), 100);
        }
    }
}

#[cfg(test)]
mod cfg_attr {
    use super::*;

    #[test]
    fn test_cfg_attr_derive_transparent_trait() {
        #[nutype(
            validate(greater_or_equal = 0, less_or_equal = 100),
            derive(Debug, PartialEq),
            cfg_attr(test, derive(Clone, Copy))
        )]
        pub struct Percent(i32);

        let p = Percent::try_new(50).unwrap();
        let p2 = p;
        let p3 = p;
        assert_eq!(p2, p3);
    }

    #[test]
    fn test_cfg_attr_derive_irregular_trait() {
        #[nutype(
            validate(greater_or_equal = 1),
            derive(Debug),
            cfg_attr(test, derive(Display, AsRef))
        )]
        pub struct PositiveInt(i64);

        let val = PositiveInt::try_new(42).unwrap();
        assert_eq!(format!("{val}"), "42");
        let inner: &i64 = val.as_ref();
        assert_eq!(*inner, 42);
    }

    #[test]
    fn test_cfg_attr_without_validation() {
        #[nutype(derive(Debug, PartialEq), cfg_attr(test, derive(Clone, Copy, Into)))]
        pub struct Count(u32);

        let c = Count::new(10);
        let c2 = c;
        let val: u32 = c2.into();
        assert_eq!(val, 10);
    }

    #[test]
    fn test_cfg_attr_derive_from_str() {
        // Conditional FromStr on integer type
        #[nutype(
            validate(greater_or_equal = 1),
            derive(Debug),
            cfg_attr(test, derive(FromStr))
        )]
        pub struct PositiveNum(i32);

        let val: PositiveNum = "42".parse().unwrap();
        assert_eq!(val.into_inner(), 42);

        // Invalid parse (not a number)
        assert!("abc".parse::<PositiveNum>().is_err());

        // Valid parse but fails validation
        assert!("0".parse::<PositiveNum>().is_err());

        // Verify ParseError type is accessible by name (re-exported)
        let err = "abc".parse::<PositiveNum>().unwrap_err();
        assert!(matches!(err, PositiveNumParseError::Parse(_)));

        let err = "0".parse::<PositiveNum>().unwrap_err();
        assert!(matches!(err, PositiveNumParseError::Validate(_)));
    }

    #[test]
    fn test_cfg_attr_derive_default() {
        // Conditional Default with unconditional default value
        #[nutype(
            derive(Debug, PartialEq),
            default = 10,
            cfg_attr(test, derive(Default))
        )]
        pub struct DefNum(i32);

        let val = DefNum::default();
        assert_eq!(val, DefNum::new(10));
    }

    #[test]
    fn test_cfg_attr_complex_predicate() {
        // Complex cfg predicate with all(...)
        #[nutype(
            derive(Debug),
            cfg_attr(all(test, target_pointer_width = "64"), derive(Clone, Copy))
        )]
        pub struct Width(u64);

        let w = Width::new(100);
        #[cfg(all(test, target_pointer_width = "64"))]
        {
            let w2 = w;
            let _w3 = w2;
        }
        let _ = w;
    }

    #[test]
    fn test_cfg_attr_cross_predicate_traits() {
        // PartialEq unconditional, Eq conditional — should work when predicate is true
        #[nutype(derive(Debug, PartialEq), cfg_attr(test, derive(Eq)))]
        pub struct Level(i32);

        let a = Level::new(5);
        let b = Level::new(5);
        assert_eq!(a, b);
    }
}
