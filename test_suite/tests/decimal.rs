#![cfg(feature = "rust_decimal")]
//! Integration tests for `rust_decimal::Decimal` as an inner type.
//! Run with: `cargo test -p test_suite --features rust_decimal`

use core::str::FromStr;
use nutype::nutype;
use rust_decimal::Decimal;

/// Small helper to build a `Decimal` from a string literal in tests.
fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

mod detection {
    use super::*;

    // All three spellings of the inner type must be detected.
    #[nutype(derive(Debug))]
    pub struct A(Decimal);

    #[nutype(derive(Debug))]
    pub struct B(rust_decimal::Decimal);

    #[nutype(derive(Debug))]
    pub struct C(::rust_decimal::Decimal);

    #[test]
    fn all_forms_compile() {
        let _ = A::new(d("1"));
        let _ = B::new(d("2"));
        let _ = C::new(d("3"));
    }
}

mod bounds {
    use super::*;

    #[nutype(
        validate(greater_or_equal = 0, less_or_equal = 100),
        derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display)
    )]
    pub struct Percentage(Decimal);

    #[test]
    fn accepts_in_range() {
        assert_eq!(
            Percentage::try_new(d("42.5")).unwrap().into_inner(),
            d("42.5")
        );
        assert_eq!(Percentage::try_new(d("0")).unwrap().into_inner(), d("0"));
        assert_eq!(
            Percentage::try_new(d("100")).unwrap().into_inner(),
            d("100")
        );
    }

    #[test]
    fn rejects_out_of_range() {
        assert_eq!(
            Percentage::try_new(d("150")),
            Err(PercentageError::LessOrEqualViolated)
        );
        assert_eq!(
            Percentage::try_new(d("-0.01")),
            Err(PercentageError::GreaterOrEqualViolated)
        );
    }

    #[nutype(
        validate(greater = 0, less = 10),
        derive(Debug, Clone, Copy, PartialEq)
    )]
    pub struct Exclusive(Decimal);

    #[test]
    fn exclusive_bounds() {
        assert_eq!(
            Exclusive::try_new(d("0")),
            Err(ExclusiveError::GreaterViolated)
        );
        assert_eq!(
            Exclusive::try_new(d("10")),
            Err(ExclusiveError::LessViolated)
        );
        assert!(Exclusive::try_new(d("5")).is_ok());
    }

    // Scale-insensitive equality of bounds: `0.50` and `0.5` are the same.
    #[nutype(
        validate(greater_or_equal = 0.50),
        derive(Debug, Clone, Copy, PartialEq)
    )]
    pub struct Scaled(Decimal);

    #[test]
    fn scale_insensitive_bound() {
        assert!(Scaled::try_new(d("0.5")).is_ok());
        assert_eq!(
            Scaled::try_new(d("0.49")),
            Err(ScaledError::GreaterOrEqualViolated)
        );
    }

    // A bound that does not fit in i64, proving the i128 `from_i128_with_scale`
    // emission is genuinely needed.
    #[nutype(
        validate(greater = 100000000000000000000),
        derive(Debug, Clone, Copy, PartialEq)
    )]
    pub struct Big(Decimal);

    #[test]
    fn out_of_i64_range_bound() {
        assert_eq!(Big::try_new(d("1")), Err(BigError::GreaterViolated));
        assert!(Big::try_new(d("100000000000000000001")).is_ok());
    }
}

mod sanitizers {
    use super::*;

    #[nutype(
        sanitize(with = |d: Decimal| d.round_dp(2)),
        validate(greater_or_equal = 0),
        derive(Debug, Clone, Copy, PartialEq),
    )]
    pub struct Money(Decimal);

    #[test]
    fn sanitizer_runs_before_validation() {
        assert_eq!(Money::try_new(d("9.999")).unwrap().into_inner(), d("10.00"));
    }

    // An *untyped* sanitizer closure must compile: the macro has to inject the
    // real `Decimal` inner type, not the internal `DecimalLit` literal type.
    #[nutype(
        sanitize(with = |d| d.round_dp(2)),
        derive(Debug, Clone, Copy, PartialEq),
    )]
    pub struct UntypedMoney(Decimal);

    #[test]
    fn untyped_sanitizer_closure() {
        assert_eq!(UntypedMoney::new(d("9.999")).into_inner(), d("10.00"));
    }
}

mod predicate {
    use super::*;

    #[nutype(
        validate(predicate = |d: &Decimal| d.scale() <= 2),
        derive(Debug, Clone, Copy, PartialEq),
    )]
    pub struct Price(Decimal);

    #[test]
    fn predicate_validation() {
        assert!(Price::try_new(d("9.99")).is_ok());
        assert_eq!(
            Price::try_new(d("9.999")),
            Err(PriceError::PredicateViolated)
        );
    }

    // An *untyped* predicate closure must compile: the macro has to inject the
    // real `&Decimal` inner type, not the internal `DecimalLit` literal type.
    #[nutype(
        validate(predicate = |d| d.scale() <= 2),
        derive(Debug, Clone, Copy, PartialEq),
    )]
    pub struct UntypedPrice(Decimal);

    #[test]
    fn untyped_predicate_closure() {
        assert!(UntypedPrice::try_new(d("9.99")).is_ok());
        assert_eq!(
            UntypedPrice::try_new(d("9.999")),
            Err(UntypedPriceError::PredicateViolated)
        );
    }
}

mod derives {
    use super::*;
    use std::collections::HashSet;

    #[nutype(derive(
        Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, FromStr, AsRef, Deref,
        Into, From, Borrow
    ))]
    pub struct Plain(Decimal);

    #[test]
    fn from_and_into() {
        let p = Plain::from(d("1.5"));
        let inner: Decimal = p.into();
        assert_eq!(inner, d("1.5"));
    }

    #[test]
    fn as_ref_and_deref() {
        let p = Plain::new(d("2.5"));
        let r: &Decimal = p.as_ref();
        assert_eq!(*r, d("2.5"));
        assert_eq!(*p, d("2.5"));
    }

    #[test]
    fn ord_eq_hash() {
        // Eq/Ord/Hash all work for Decimal with no `finite`-style caveat.
        assert!(Plain::new(d("1")) < Plain::new(d("2")));
        let mut set = HashSet::new();
        set.insert(Plain::new(d("1.0")));
        // 1.0 == 1.00 for Decimal, and Hash is consistent with Eq.
        assert!(set.contains(&Plain::new(d("1.00"))));
    }

    #[test]
    fn display_and_from_str() {
        let p = Plain::new(d("3.14"));
        assert_eq!(p.to_string(), "3.14");
        let parsed = Plain::from_str("3.14").unwrap();
        assert_eq!(parsed, p);
    }

    #[nutype(
        validate(greater_or_equal = 0),
        derive(Debug, Clone, Copy, PartialEq, TryFrom)
    )]
    pub struct NonNegative(Decimal);

    #[test]
    fn try_from_with_validation() {
        assert!(NonNegative::try_from(d("1")).is_ok());
        assert!(NonNegative::try_from(d("-1")).is_err());
    }
}

mod default_trait {
    use super::*;
    use rust_decimal_macros::dec;

    // Constant / path expression form.
    #[nutype(
        derive(Debug, Clone, Copy, PartialEq, Default),
        default = Decimal::ZERO,
    )]
    pub struct Quantity(Decimal);

    // Bare integer literal form.
    #[nutype(derive(Debug, Clone, Copy, PartialEq, Default), default = 0)]
    pub struct Count(Decimal);

    // Bare float literal form.
    #[nutype(derive(Debug, Clone, Copy, PartialEq, Default), default = 0.5)]
    pub struct Ratio(Decimal);

    // Negative float literal form, combined with validation.
    #[nutype(
        validate(greater_or_equal = -10),
        derive(Debug, Clone, Copy, PartialEq, Default),
        default = -1.5,
    )]
    pub struct Offset(Decimal);

    // Negative integer literal form.
    #[nutype(derive(Debug, Clone, Copy, PartialEq, Default), default = -7)]
    pub struct Temperature(Decimal);

    // `dec!(...)` macro form.
    #[nutype(derive(Debug, Clone, Copy, PartialEq, Default), default = dec!(0.25))]
    pub struct Fraction(Decimal);

    // Out-of-i64-range literal default, to confirm the i128 path is used.
    #[nutype(
        derive(Debug, Clone, Copy, PartialEq, Default),
        default = 100000000000000000000
    )]
    pub struct Huge(Decimal);

    #[test]
    fn default_value() {
        assert_eq!(Quantity::default().into_inner(), Decimal::ZERO);
        assert_eq!(Count::default().into_inner(), d("0"));
        assert_eq!(Ratio::default().into_inner(), d("0.5"));
        assert_eq!(Offset::default().into_inner(), d("-1.5"));
        assert_eq!(Temperature::default().into_inner(), d("-7"));
        assert_eq!(Fraction::default().into_inner(), dec!(0.25));
        assert_eq!(Huge::default().into_inner(), d("100000000000000000000"));
    }
}

#[cfg(feature = "serde")]
mod serde_support {
    use super::*;

    #[nutype(
        validate(greater_or_equal = 0, less_or_equal = 100),
        derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)
    )]
    pub struct Percent(Decimal);

    #[test]
    fn round_trip() {
        let p = Percent::try_new(d("12.5")).unwrap();
        let json = serde_json::to_string(&p).unwrap();
        let back: Percent = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn deserialize_rejects_out_of_range() {
        let result: Result<Percent, _> = serde_json::from_str("150");
        assert!(result.is_err());
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary_support {
    use super::*;
    use arbitrary::{Arbitrary, Unstructured};

    #[nutype(derive(Debug, Clone, Copy, PartialEq, Arbitrary))]
    pub struct AnyDecimal(Decimal);

    #[nutype(
        validate(greater_or_equal = 0, less_or_equal = 100),
        derive(Debug, Clone, Copy, PartialEq, PartialOrd, Arbitrary)
    )]
    pub struct BoundedPercentage(Decimal);

    #[nutype(
        validate(greater_or_equal = 0),
        derive(Debug, Clone, Copy, PartialEq, PartialOrd, Arbitrary)
    )]
    pub struct LowerBounded(Decimal);

    #[test]
    fn unvalidated_arbitrary() {
        let data = [0xABu8; 64];
        let mut u = Unstructured::new(&data);
        let _ = AnyDecimal::arbitrary(&mut u).unwrap();
    }

    #[test]
    fn bounded_arbitrary_stays_in_range() {
        // Many seeds must always produce valid values inside [0, 100].
        for seed in 0u8..=255 {
            let data = [seed; 64];
            let mut u = Unstructured::new(&data);
            if let Ok(value) = BoundedPercentage::arbitrary(&mut u) {
                let inner = value.into_inner();
                assert!(
                    inner >= d("0") && inner <= d("100"),
                    "out of range: {inner}"
                );
            }
        }
    }

    #[test]
    fn lower_bounded_arbitrary_stays_in_range() {
        for seed in 0u8..=255 {
            let data = [seed; 64];
            let mut u = Unstructured::new(&data);
            if let Ok(value) = LowerBounded::arbitrary(&mut u) {
                assert!(value.into_inner() >= d("0"));
            }
        }
    }
}

mod constructor_visibility {
    use super::*;

    #[nutype(
        validate(greater_or_equal = 0),
        constructor(visibility = private),
        derive(Debug, Clone, Copy, PartialEq),
    )]
    pub struct Internal(Decimal);

    impl Internal {
        pub fn make(value: Decimal) -> Option<Self> {
            Self::try_new(value).ok()
        }
    }

    #[test]
    fn private_constructor() {
        assert!(Internal::make(d("1")).is_some());
        assert!(Internal::make(d("-1")).is_none());
    }
}
