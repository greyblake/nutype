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
        #[nutype]
        #[derive(From)]
        struct Age(f64);

        assert_eq!(Age::from(17.0).into_inner(), 17.0);
    }
}

#[cfg(test)]
mod validators {
    use super::*;

    #[test]
    fn test_min() {
        #[nutype(validate(min = 18.0))]
        #[derive(*)]
        struct Age(f32);

        assert_eq!(Age::new(17.0).unwrap_err(), AgeError::TooSmall);
        assert_eq!(Age::new(18.0).unwrap().into_inner(), 18.0);
    }

    #[test]
    fn test_max() {
        #[nutype(validate(max = 99.0))]
        #[derive(*)]
        struct Age(f32);

        assert_eq!(Age::new(100.0).unwrap_err(), AgeError::TooBig);
        assert_eq!(Age::new(99.0).unwrap().into_inner(), 99.0);
    }

    #[test]
    fn test_min_and_max() {
        #[nutype(validate(min = 18.0, max = 99.0))]
        #[derive(*)]
        struct Age(f32);

        assert_eq!(Age::new(17.9).unwrap_err(), AgeError::TooSmall);
        assert_eq!(Age::new(99.1).unwrap_err(), AgeError::TooBig);
        assert_eq!(Age::new(25.0).unwrap().into_inner(), 25.0);
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(validate(with = |&c: &f32| c >= 0.0 && c <= 100.0 ))]
            #[derive(*)]
            pub struct Cent(f32);

            assert_eq!(Cent::new(-0.1), Err(CentError::Invalid));
            assert_eq!(Cent::new(100.1), Err(CentError::Invalid));
            assert_eq!(Cent::new(100.0).unwrap().into_inner(), 100.0);
            assert_eq!(Cent::new(0.0).unwrap().into_inner(), 0.0);
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(validate(with = |&c| c >= 0.0 && c <= 100.0 ))]
            #[derive(*)]
            pub struct Cent(f32);

            assert_eq!(Cent::new(-0.1), Err(CentError::Invalid));
            assert_eq!(Cent::new(100.1), Err(CentError::Invalid));
            assert_eq!(Cent::new(100.0).unwrap().into_inner(), 100.0);
            assert_eq!(Cent::new(0.0).unwrap().into_inner(), 0.0);
        }

        fn is_cent_valid(&val: &f32) -> bool {
            val >= 0.0 && val <= 100.0
        }

        #[test]
        fn test_with_function() {
            #[nutype(validate(with = is_cent_valid))]
            #[derive(*)]
            pub struct Cent(f32);

            assert_eq!(Cent::new(-0.1), Err(CentError::Invalid));
            assert_eq!(Cent::new(100.1), Err(CentError::Invalid));
            assert_eq!(Cent::new(100.0).unwrap().into_inner(), 100.0);
            assert_eq!(Cent::new(0.0).unwrap().into_inner(), 0.0);
        }
    }

    #[test]
    fn test_try_from_trait() {
        #[nutype(validate(min = 18.0))]
        #[derive(*)]
        struct Age(f64);

        assert_eq!(Age::try_from(17.9).unwrap_err(), AgeError::TooSmall);
        assert_eq!(Age::try_from(18.0).unwrap().into_inner(), 18.0);
    }

    #[cfg(test)]
    mod error {
        use super::*;

        #[test]
        fn test_error_display() {
            #[nutype(validate(min = 0.0))]
            #[derive(*)]
            struct Percentage(f64);

            let err = Percentage::try_from(-0.1).unwrap_err();

            assert_eq!(err.to_string(), "too small");
        }
    }
}

#[cfg(test)]
mod types {
    use super::*;

    #[test]
    fn test_f32_validate() {
        #[nutype(validate(min = 0.0, max = 100))]
        #[derive(Debug, PartialEq)]
        pub struct Width(f32);

        assert_eq!(Width::new(-0.0001), Err(WidthError::TooSmall));
        assert_eq!(Width::new(100.0001), Err(WidthError::TooBig));
        assert!(Width::new(0.0).is_ok());
        assert!(Width::new(100.0).is_ok());
    }

    #[test]
    fn test_f64_validate() {
        #[nutype(validate(min = 0.0, max = 100))]
        #[derive(Debug, PartialEq)]
        pub struct Width(f64);

        assert_eq!(Width::new(-0.0001), Err(WidthError::TooSmall));
        assert_eq!(Width::new(100.0001), Err(WidthError::TooBig));

        assert_eq!(Width::new(0.0).unwrap().into_inner(), 0.0);

        let w: f64 = Width::new(100.0).unwrap().into_inner();
        assert_eq!(w, 100.0);
    }

    #[test]
    fn test_f64_negative() {
        #[nutype(
            sanitize(with = |n| n.clamp(-200.25, -5.0))
            validate(min = -100.25, max = -50.1)
        )]
        #[derive(*)]
        pub struct Balance(f64);

        assert_eq!(Balance::new(-300.0), Err(BalanceError::TooSmall));
        assert_eq!(Balance::new(-4.0), Err(BalanceError::TooBig));

        let balance = Balance::new(-100.24).unwrap();
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
        #[nutype]
        #[derive(Debug, From, FromStr, Borrow, Clone, Copy)]
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
        #[nutype(validate(max = 100.0))]
        #[derive(Debug, TryFrom, FromStr, Borrow, Clone, Copy)]
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
        #[nutype]
        #[derive(Into)]
        pub struct Size(f64);

        let size = Size::new(35.7);
        let size: f64 = size.into();
        assert_eq!(size, 35.7);
    }

    #[test]
    fn test_trait_from() {
        #[nutype]
        #[derive(From)]
        pub struct Size(f64);

        let size = Size::from(35.7);
        assert_eq!(size.into_inner(), 35.7);
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype]
        #[derive(AsRef)]
        pub struct Weight(f32);

        let weight = Weight::new(72.650);
        let weight_ref: &f32 = weight.as_ref();
        assert_eq!(weight_ref, &72.650);
    }

    #[test]
    fn test_trait_borrow() {
        use std::borrow::Borrow;

        #[nutype]
        #[derive(Borrow)]
        pub struct Age(f64);

        let age = Age::new(32.0);
        let age_borrowed: &f64 = age.borrow();
        assert_eq!(age_borrowed, &32.0);
    }

    #[test]
    fn test_trait_try_from() {
        #[nutype(validate(max = 12.34))]
        #[derive(Debug, TryFrom)]
        pub struct Dist(f64);

        let dist = Dist::try_from(12.34).unwrap();
        assert_eq!(dist.into_inner(), 12.34);

        let error = Dist::try_from(12.35).unwrap_err();
        assert_eq!(error, DistError::TooBig);
    }

    #[test]
    fn test_trait_from_str_without_validation() {
        #[nutype]
        #[derive(Debug, FromStr)]
        pub struct Dist(f64);

        let dist: Dist = "33.4".parse().unwrap();
        assert_eq!(dist.into_inner(), 33.4);

        let err: DistParseError = "foobar".parse::<Dist>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Dist: invalid float literal"
        );
    }

    #[test]
    fn test_trait_from_str_with_validation() {
        #[nutype(validate(max = 12.34))]
        #[derive(Debug, FromStr)]
        pub struct Dist(f64);

        // Happy path
        let dist: Dist = "11.4".parse().unwrap();
        assert_eq!(dist.into_inner(), 11.4);

        let err: DistParseError = "foobar".parse::<Dist>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Failed to parse Dist: invalid float literal"
        );

        // Unhappy path: validation error
        let err: DistParseError = "12.35".parse::<Dist>().unwrap_err();
        assert_eq!(err.to_string(), "Failed to parse Dist: too big");
    }

    #[test]
    fn test_trait_display() {
        #[nutype]
        #[derive(Display)]
        pub struct Size(f64);

        let size = Size::new(35.7);
        assert_eq!(size.to_string(), "35.7");
    }

    #[cfg(feature = "serde1")]
    #[test]
    fn test_trait_serialize() {
        #[nutype]
        #[derive(Serialize)]
        pub struct Offset(f64);

        let offset = Offset::new(-33.5);
        let offset_json = serde_json::to_string(&offset).unwrap();
        assert_eq!(offset_json, "-33.5");
    }
}
