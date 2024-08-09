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
            #[nutype(sanitize(with = |n: f64| n.clamp(0.0, 100.0)))]
            pub struct Cent(f64);

            assert_eq!(Cent::new(-10.0).into_inner(), 0.0);
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(sanitize(with = |n| n.clamp(0.0, 100.0)))]
            pub struct Cent(f64);

            assert_eq!(Cent::new(-10.0).into_inner(), 0.0);
        }

        fn sanitize_cent(value: f64) -> f64 {
            value.clamp(0.0, 100.0)
        }

        #[test]
        fn test_with_function() {
            #[nutype(sanitize(with = sanitize_cent))]
            pub struct Cent(f64);

            assert_eq!(Cent::new(222.0).into_inner(), 100.0);
        }
    }

    #[test]
    fn test_from_trait() {
        #[nutype(derive(From))]
        struct Age(f64);

        assert_eq!(Age::from(17.0).into_inner(), 17.0);
    }
}

#[cfg(test)]
mod validators {
    use super::*;

    #[test]
    fn test_greater() {
        #[nutype(validate(greater = 18.0), derive(Debug))]
        struct Age(f64);

        assert_eq!(Age::try_new(18.0).unwrap_err(), AgeError::GreaterViolated);
        assert_eq!(Age::try_new(18.00001).unwrap().into_inner(), 18.00001);
    }

    #[test]
    fn test_greater_or_equal() {
        #[nutype(validate(greater_or_equal = 18.0), derive(Debug))]
        struct Age(f32);

        assert_eq!(
            Age::try_new(17.0).unwrap_err(),
            AgeError::GreaterOrEqualViolated
        );
        assert_eq!(Age::try_new(18.0).unwrap().into_inner(), 18.0);
    }

    #[test]
    fn test_less() {
        #[nutype(validate(less = 99.0), derive(Debug))]
        struct Age(f64);

        assert_eq!(Age::try_new(99.0).unwrap_err(), AgeError::LessViolated);
        assert_eq!(Age::try_new(98.99999).unwrap().into_inner(), 98.99999);
    }

    #[test]
    fn test_less_or_equal() {
        #[nutype(validate(less_or_equal = 99.0), derive(Debug))]
        struct Age(f32);

        assert_eq!(
            Age::try_new(100.0).unwrap_err(),
            AgeError::LessOrEqualViolated
        );
        assert_eq!(Age::try_new(99.0).unwrap().into_inner(), 99.0);
    }

    #[test]
    fn test_greater_or_equal_and_less_or_equal() {
        #[nutype(validate(greater_or_equal = 18.0, less_or_equal = 99.0), derive(Debug))]
        struct Age(f32);

        assert_eq!(
            Age::try_new(17.9).unwrap_err(),
            AgeError::GreaterOrEqualViolated
        );
        assert_eq!(
            Age::try_new(99.1).unwrap_err(),
            AgeError::LessOrEqualViolated
        );
        assert_eq!(Age::try_new(25.0).unwrap().into_inner(), 25.0);
    }

    #[test]
    fn test_finite_f64() {
        #[nutype(validate(finite), derive(Debug, PartialEq))]
        struct Dist(f64);

        // invalid
        assert_eq!(Dist::try_new(f64::INFINITY), Err(DistError::FiniteViolated));
        assert_eq!(
            Dist::try_new(f64::NEG_INFINITY),
            Err(DistError::FiniteViolated)
        );
        assert_eq!(Dist::try_new(f64::NAN), Err(DistError::FiniteViolated));
        assert_eq!(Dist::try_new(-1.0 / 0.0), Err(DistError::FiniteViolated));
        assert_eq!(Dist::try_new(1.0 / 0.0), Err(DistError::FiniteViolated));
        assert_eq!(Dist::try_new(0.0 / 0.0), Err(DistError::FiniteViolated));

        // valid
        assert_eq!(Dist::try_new(12.345).unwrap().into_inner(), 12.345);
        assert_eq!(Dist::try_new(-999.12).unwrap().into_inner(), -999.12);
    }

    #[test]
    fn test_finite_f32() {
        #[nutype(
            validate(finite),
            derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef)
        )]
        struct Dist(f32);

        // invalid
        assert_eq!(Dist::try_new(-1.0 / 0.0), Err(DistError::FiniteViolated));
        assert_eq!(Dist::try_new(1.0 / 0.0), Err(DistError::FiniteViolated));
        assert_eq!(Dist::try_new(0.0 / 0.0), Err(DistError::FiniteViolated));

        // valid
        assert_eq!(Dist::try_new(12.345).unwrap().into_inner(), 12.345);
        assert_eq!(Dist::try_new(-999.12).unwrap().into_inner(), -999.12);
    }

    mod when_boundaries_defined_as_constants {
        use super::*;

        // Inclusive range
        const MIN_WEIGHT: f32 = 2.0;
        const MAX_WEIGHT: f32 = 5.0;
        #[nutype(validate(greater_or_equal = MIN_WEIGHT, less_or_equal = MAX_WEIGHT), derive(Debug))]
        struct Weight(f32);

        // // Exclusive range
        const MIN_SPEED: f64 = 60.0;
        const MAX_SPEED: f64 = 130.0; // this is weird, but it's just for the sake of testing
        #[nutype(validate(greater = MIN_SPEED, less = MAX_SPEED), derive(Debug))]
        struct Speed(f64);

        #[test]
        fn test_boundaries_defined_as_constants() {
            assert_eq!(
                Weight::try_new(1.99999).unwrap_err(),
                WeightError::GreaterOrEqualViolated
            );
            assert_eq!(Weight::try_new(2.0).unwrap().into_inner(), 2.0);
            assert_eq!(
                Weight::try_new(5.00001).unwrap_err(),
                WeightError::LessOrEqualViolated
            );
            assert_eq!(Weight::try_new(5.0).unwrap().into_inner(), 5.0);

            assert_eq!(
                Speed::try_new(60.0).unwrap_err(),
                SpeedError::GreaterViolated
            );
            assert_eq!(Speed::try_new(60.00001).unwrap().into_inner(), 60.00001);
            assert_eq!(Speed::try_new(130.0).unwrap_err(), SpeedError::LessViolated);
            assert_eq!(Speed::try_new(129.99999).unwrap().into_inner(), 129.99999);
        }
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(validate(predicate = |&c: &f32| (0.0..=100.0).contains(&c) ), derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef))]
            pub struct Cent(f32);

            assert_eq!(Cent::try_new(-0.1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100.1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100.0).unwrap().into_inner(), 100.0);
            assert_eq!(Cent::try_new(0.0).unwrap().into_inner(), 0.0);
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(validate(predicate = |&c| (0.0..=100.0).contains(&c) ), derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef))]
            pub struct Cent(f32);

            assert_eq!(Cent::try_new(-0.1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100.1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100.0).unwrap().into_inner(), 100.0);
            assert_eq!(Cent::try_new(0.0).unwrap().into_inner(), 0.0);
        }

        fn is_cent_valid(&val: &f32) -> bool {
            (0.0..=100.0).contains(&val)
        }

        #[test]
        fn test_with_function() {
            #[nutype(validate(predicate = is_cent_valid), derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef))]
            pub struct Cent(f32);

            assert_eq!(Cent::try_new(-0.1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100.1), Err(CentError::PredicateViolated));
            assert_eq!(Cent::try_new(100.0).unwrap().into_inner(), 100.0);
            assert_eq!(Cent::try_new(0.0).unwrap().into_inner(), 0.0);
        }
    }

    #[test]
    fn test_try_from_trait() {
        #[nutype(
            validate(greater_or_equal = 18.0),
            derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef)
        )]
        struct Age(f64);

        assert_eq!(
            Age::try_from(17.9).unwrap_err(),
            AgeError::GreaterOrEqualViolated
        );
        assert_eq!(Age::try_from(18.0).unwrap().into_inner(), 18.0);
    }

    #[test]
    fn test_try_from_trait_without_validation() {
        #[nutype(derive(Debug, PartialEq, TryFrom))]
        struct Age(f64);

        assert_eq!(Age::try_from(78.8).unwrap().into_inner(), 78.8);
    }

    #[cfg(test)]
    mod error {
        use super::*;

        #[test]
        fn test_error_display() {
            #[nutype(
                validate(greater_or_equal = 0.0),
                derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef)
            )]
            struct Percentage(f64);

            let err = Percentage::try_from(-0.1).unwrap_err();

            assert_eq!(
                err.to_string(),
                "Percentage is too small. The value must be greater or equal to 0.0."
            );
        }
    }
}

#[cfg(test)]
mod types {
    use super::*;

    #[test]
    fn test_f32_validate() {
        #[nutype(
            validate(greater_or_equal = 0.0, less_or_equal = 100),
            derive(Debug, PartialEq)
        )]
        pub struct Width(f32);

        assert_eq!(
            Width::try_new(-0.0001),
            Err(WidthError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Width::try_new(100.0001),
            Err(WidthError::LessOrEqualViolated)
        );
        assert!(Width::try_new(0.0).is_ok());
        assert!(Width::try_new(100.0).is_ok());
    }

    #[test]
    fn test_f64_validate() {
        #[nutype(
            validate(greater_or_equal = 0.0, less_or_equal = 100),
            derive(Debug, PartialEq)
        )]
        pub struct Width(f64);

        assert_eq!(
            Width::try_new(-0.0001),
            Err(WidthError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Width::try_new(100.0001),
            Err(WidthError::LessOrEqualViolated)
        );

        assert_eq!(Width::try_new(0.0).unwrap().into_inner(), 0.0);

        let w: f64 = Width::try_new(100.0).unwrap().into_inner();
        assert_eq!(w, 100.0);
    }

    #[test]
    fn test_f64_negative() {
        #[nutype(
            sanitize(with = |n| n.clamp(-200.25, -5.0)),
            validate(greater_or_equal = -100.25, less_or_equal = -50.1),
            derive(TryFrom, Debug, Clone, Copy, PartialEq, PartialOrd, FromStr, AsRef)
        )]
        pub struct Balance(f64);

        assert_eq!(
            Balance::try_new(-300.0),
            Err(BalanceError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Balance::try_new(-4.0),
            Err(BalanceError::LessOrEqualViolated)
        );

        let balance = Balance::try_new(-100.24).unwrap();
        assert_eq!(balance.into_inner(), -100.24);
    }
}

#[cfg(test)]
mod visibility {
    mod encapsulated {
        use nutype::nutype;

        #[nutype(sanitize(with = |n: f32| n.clamp(0.0, 100.0)))]
        pub struct Percentage(f32);
    }

    #[test]
    fn test_public_visibility() {
        let percentage = encapsulated::Percentage::new(133.0);
        assert_eq!(percentage.into_inner(), 100.0);
    }
}

#[cfg(test)]
mod traits {
    use super::*;
    use test_suite::test_helpers::traits::*;

    #[test]
    fn test_without_validation() {
        #[nutype(derive(Debug, From, FromStr, Borrow, Clone, Copy))]
        pub struct Dist(f32);

        should_implement_debug::<Dist>();
        should_implement_from::<Dist, f32>();
        should_implement_from_str::<Dist>();
        should_implement_borrow::<Dist, f32>();
        should_implement_clone::<Dist>();
        should_implement_copy::<Dist>();
    }

    #[test]
    fn test_with_validaiton() {
        #[nutype(
            validate(less_or_equal = 100.0),
            derive(Debug, TryFrom, FromStr, Borrow, Clone, Copy)
        )]
        pub struct Dist(f64);

        should_implement_debug::<Dist>();
        should_implement_try_from::<Dist, f64>();
        should_implement_from_str::<Dist>();
        should_implement_borrow::<Dist, f64>();
        should_implement_clone::<Dist>();
        should_implement_copy::<Dist>();
    }

    #[test]
    fn test_trait_into() {
        #[nutype(derive(Into))]
        pub struct Size(f64);

        let size = Size::new(35.7);
        let size: f64 = size.into();
        assert_eq!(size, 35.7);
    }

    #[test]
    fn test_trait_from() {
        #[nutype(derive(From))]
        pub struct Size(f64);

        let size = Size::from(35.7);
        assert_eq!(size.into_inner(), 35.7);
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype(derive(AsRef))]
        pub struct Weight(f32);

        let weight = Weight::new(72.650);
        let weight_ref: &f32 = weight.as_ref();
        assert_eq!(weight_ref, &72.650);
    }

    #[test]
    fn test_trait_deref() {
        #[nutype(derive(Deref))]
        pub struct Number(f64);

        let magic = Number::new(42.0);
        assert_eq!(*magic, 42.0);
    }

    #[test]
    fn test_trait_borrow() {
        use std::borrow::Borrow;

        #[nutype(derive(Borrow))]
        pub struct Age(f64);

        let age = Age::new(32.0);
        let age_borrowed: &f64 = age.borrow();
        assert_eq!(age_borrowed, &32.0);
    }

    #[test]
    fn test_trait_try_from() {
        #[nutype(validate(less_or_equal = 12.34), derive(Debug, TryFrom))]
        pub struct Dist(f64);

        let dist = Dist::try_from(12.34).unwrap();
        assert_eq!(dist.into_inner(), 12.34);

        let error = Dist::try_from(12.35).unwrap_err();
        assert_eq!(error, DistError::LessOrEqualViolated);
    }

    #[test]
    fn test_trait_from_str_without_validation() {
        #[nutype(derive(Debug, FromStr))]
        pub struct Dist(f64);

        let dist: Dist = "33.4".parse().unwrap();
        assert_eq!(dist.into_inner(), 33.4);

        let err: DistParseError = "foobar".parse::<Dist>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Dist: ParseFloatError { kind: Invalid }"
        );
    }

    #[test]
    fn test_trait_from_str_with_validation() {
        #[nutype(validate(less_or_equal = 12.34), derive(Debug, FromStr))]
        pub struct Dist(f64);

        // Happy path
        let dist: Dist = "11.4".parse().unwrap();
        assert_eq!(dist.into_inner(), 11.4);

        let err: DistParseError = "foobar".parse::<Dist>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Dist: ParseFloatError { kind: Invalid }"
        );

        // Unhappy path: validation error
        let err: DistParseError = "12.35".parse::<Dist>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Dist: Dist is too big. The value must be less than 12.34."
        );
    }

    #[test]
    fn test_trait_display() {
        #[nutype(derive(Display))]
        pub struct Size(f64);

        let size = Size::new(35.7);
        assert_eq!(size.to_string(), "35.7");
    }

    #[test]
    fn test_trait_eq() {
        #[nutype(validate(finite), derive(PartialEq, Eq, Debug))]
        pub struct Size(f64);

        should_implement_eq::<Size>();

        let size1 = Size::try_new(35.7).unwrap();
        let size2 = Size::try_new(357.0 / 10.0).unwrap();
        assert_eq!(size1, size2);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::cmp::Ordering;

        #[test]
        fn test_trait_ord_f32() {
            #[nutype(validate(finite), derive(PartialEq, Eq, PartialOrd, Ord))]
            pub struct Size(f32);

            let a: Size = Size::try_new(2.5).unwrap();
            let b: Size = Size::try_new(3.3).unwrap();
            let c: Size = Size::try_new(3.3).unwrap();

            assert_eq!(a.cmp(&b), Ordering::Less);
            assert_eq!(b.cmp(&a), Ordering::Greater);
            assert_eq!(b.cmp(&c), Ordering::Equal);
        }

        #[test]
        fn test_trait_ord_f64() {
            #[nutype(validate(finite), derive(PartialEq, Eq, PartialOrd, Ord))]
            pub struct Size(f64);

            let a: Size = Size::try_new(2.5).unwrap();
            let b: Size = Size::try_new(3.3).unwrap();
            let c: Size = Size::try_new(3.3).unwrap();

            assert_eq!(a.cmp(&b), Ordering::Less);
            assert_eq!(b.cmp(&a), Ordering::Greater);
            assert_eq!(b.cmp(&c), Ordering::Equal);
        }

        #[test]
        fn test_sort() {
            #[nutype(validate(finite), derive(PartialEq, Eq, PartialOrd, Ord))]
            pub struct Size(f64);

            let initial_raw_sizes = vec![5.5, 2.0, 15.0, 44.5, 3.5];
            let mut sizes: Vec<Size> = initial_raw_sizes
                .into_iter()
                .map(|s| Size::try_new(s).unwrap())
                .collect();
            sizes.sort();
            let sorted_raw_sizes: Vec<f64> = sizes.into_iter().map(Size::into_inner).collect();
            assert_eq!(sorted_raw_sizes, vec![2.0, 3.5, 5.5, 15.0, 44.5],);
        }

        #[cfg(test)]
        mod prop_tests {
            use super::*;
            use arbitrary::{Error, Unstructured};

            #[test]
            fn cmp_never_panics_f32() {
                #[nutype(validate(finite), derive(PartialEq, Eq, PartialOrd, Ord))]
                pub struct Size(f32);

                fn prop(u: &mut Unstructured<'_>) -> arbitrary::Result<()> {
                    let raw_a: f32 = u.arbitrary()?;
                    let a = Size::try_new(raw_a).map_err(|_| Error::IncorrectFormat)?;
                    let raw_b: f32 = u.arbitrary()?;
                    let b = Size::try_new(raw_b).map_err(|_| Error::IncorrectFormat)?;
                    let _ = a.cmp(&b);
                    let _ = b.cmp(&a);
                    Ok(())
                }

                arbtest::builder().run(prop);
            }

            #[test]
            fn cmp_never_panics_f64() {
                #[nutype(validate(finite), derive(PartialEq, Eq, PartialOrd, Ord))]
                pub struct Size(f64);

                fn prop(u: &mut Unstructured<'_>) -> arbitrary::Result<()> {
                    let raw_a: f64 = u.arbitrary()?;
                    let a = Size::try_new(raw_a).map_err(|_| Error::IncorrectFormat)?;
                    let raw_b: f64 = u.arbitrary()?;
                    let b = Size::try_new(raw_b).map_err(|_| Error::IncorrectFormat)?;
                    let _ = a.cmp(&b);
                    let _ = b.cmp(&a);
                    Ok(())
                }

                arbtest::builder().run(prop);
            }
        }
    }

    #[cfg(feature = "serde")]
    mod serialization {
        use super::*;

        mod json_format {
            use super::*;

            #[test]
            fn test_trait_serialize() {
                #[nutype(derive(Serialize))]
                pub struct Offset(f64);

                let offset = Offset::new(-33.5);
                let offset_json = serde_json::to_string(&offset).unwrap();
                assert_eq!(offset_json, "-33.5");
            }

            #[test]
            fn test_trait_deserialize_without_validation() {
                #[nutype(derive(Deserialize))]
                pub struct Offset(f64);

                {
                    let res: Result<Offset, _> = serde_json::from_str("three");
                    assert!(res.is_err());
                }

                {
                    let offset: Offset = serde_json::from_str("-259.28").unwrap();
                    assert_eq!(offset.into_inner(), -259.28);
                }
            }

            #[test]
            fn test_trait_deserialize_with_validation() {
                #[nutype(validate(greater_or_equal = 13.3), derive(Deserialize))]
                pub struct Offset(f32);

                {
                    let res: Result<Offset, _> = serde_json::from_str("three");
                    assert!(res.is_err());
                }

                {
                    let res: Result<Offset, _> = serde_json::from_str("13.2");
                    assert!(res.is_err());
                }

                {
                    let offset: Offset = serde_json::from_str("13.3").unwrap();
                    assert_eq!(offset.into_inner(), 13.3);
                }
            }
        }

        mod ron_format {
            use super::*;

            #[test]
            fn test_ron_roundtrip() {
                #[nutype(derive(Serialize, Deserialize, PartialEq, Debug))]
                pub struct Weight(f64);

                let weight = Weight::new(33.5);

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
                pub struct Weight(f64);

                let weight = Weight::new(77.7);

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
            #[nutype(default = 13.0, derive(Default))]
            pub struct Number(f32);

            assert_eq!(Number::default().into_inner(), 13.0);
        }

        #[test]
        fn test_default_with_validation_when_valid() {
            #[nutype(validate(less_or_equal = 20.0), default = 13.0, derive(Default))]
            pub struct Number(f64);

            assert_eq!(Number::default().into_inner(), 13.0);
        }

        #[test]
        #[should_panic(expected = "Default value for type `Number` is invalid")]
        fn test_default_with_validation_when_invalid() {
            #[nutype(validate(less_or_equal = 20.0), default = 20.1, derive(Default))]
            pub struct Number(f64);

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
        #[nutype(new_unchecked, validate(greater_or_equal = 50.0))]
        pub struct Dist(f64);

        let dist = unsafe { Dist::new_unchecked(3.0) };
        assert_eq!(dist.into_inner(), 3.0);
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
        pub struct ProductWeight(f64);

        assert_eq!(ProductWeight::schema_name(), "ProductWeight");
        // Make sure it compiles
        let _schema = schema_for!(ProductWeight);
    }
}
