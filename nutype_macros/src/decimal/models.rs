use core::{cmp::Ordering, str::FromStr};

use kinded::Kinded;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::common::models::{
    Guard, RawGuard, SpannedItem, TypeTrait, TypedCustomFunction, ValueOrExpr,
    impl_numeric_bound_on_vec_of, impl_numeric_bound_validator,
};

// Literal value
//

/// A decimal literal value parsed at compile time.
///
/// This is a thin wrapper around a real `rust_decimal::Decimal`. The wrapper
/// exists only to satisfy the orphan rule for `ToTokens` (we cannot
/// `impl ToTokens for rust_decimal::Decimal`). All parsing and comparison
/// delegate to `Decimal`.
#[derive(Debug, Clone)]
pub struct DecimalLit(rust_decimal::Decimal);

impl FromStr for DecimalLit {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Delegate to rust_decimal's own parser: we accept exactly what it
        // accepts (after the syn `Lit` layer has already lexed the literal).
        rust_decimal::Decimal::from_str(s).map(DecimalLit)
    }
}

impl PartialEq for DecimalLit {
    fn eq(&self, other: &Self) -> bool {
        // `Decimal` already treats `0.5 == 0.50` as equal.
        self.0 == other.0
    }
}

impl Eq for DecimalLit {}

impl PartialOrd for DecimalLit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DecimalLit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl ToTokens for DecimalLit {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        // Reconstruct the value losslessly from its components. `mantissa()`
        // and `scale()` of an existing `Decimal` are always in range, so
        // `from_i128_with_scale` never panics here.
        let mantissa: i128 = self.0.mantissa();
        let scale: u32 = self.0.scale();
        quote!(
            ::rust_decimal::Decimal::from_i128_with_scale(#mantissa, #scale)
        )
        .to_tokens(token_stream);
    }
}

/// Marker trait, mirroring `IntegerType`/`FloatType`. There is exactly one
/// decimal value type (`DecimalLit`), but keeping the newtype generic over `T`
/// lets us reuse the same machinery as integers and floats.
pub trait DecimalType {}

impl DecimalType for DecimalLit {}

// Inner type
//

/// The inner type of a decimal-based newtype: `rust_decimal::Decimal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalInnerType;

impl ToTokens for DecimalInnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        quote!(::rust_decimal::Decimal).to_tokens(token_stream);
    }
}

impl ::core::fmt::Display for DecimalInnerType {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        "rust_decimal::Decimal".fmt(f)
    }
}

// Sanitizer
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum DecimalSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(core::marker::PhantomData<T>),
}

pub type SpannedDecimalSanitizer<T> = SpannedItem<DecimalSanitizer<T>>;

// Validator
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum DecimalValidator<T> {
    Greater(ValueOrExpr<T>),
    GreaterOrEqual(ValueOrExpr<T>),
    Less(ValueOrExpr<T>),
    LessOrEqual(ValueOrExpr<T>),
    Predicate(TypedCustomFunction),
}

impl_numeric_bound_validator!(DecimalValidator);
impl_numeric_bound_on_vec_of!(DecimalValidator);

pub type SpannedDecimalValidator<T> = SpannedItem<DecimalValidator<T>>;

// Traits
//
// Same set as `IntegerDeriveTrait`, with three deliberate differences:
// * no `SchemarsJsonSchema` (deferred),
// * no `ValuableValuable` (`rust_decimal::Decimal` does not implement `Valuable`),
// * `ArbitraryArbitrary` is supported (requires the user to enable
//   `rust_decimal/rust-fuzz`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum DecimalDeriveTrait {
    // Standard
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    FromStr,
    AsRef,
    Into,
    From,
    TryFrom,
    Hash,
    Borrow,
    Display,
    Default,
    Deref,

    // External crates
    SerdeSerialize,
    SerdeDeserialize,
    ArbitraryArbitrary,
}

impl TypeTrait for DecimalDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &DecimalDeriveTrait::FromStr
    }
    fn is_default(&self) -> bool {
        self == &DecimalDeriveTrait::Default
    }
}

pub type DecimalRawGuard<T> = RawGuard<SpannedDecimalSanitizer<T>, SpannedDecimalValidator<T>>;
pub type DecimalGuard<T> = Guard<DecimalSanitizer<T>, DecimalValidator<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn lit(s: &str) -> DecimalLit {
        s.parse::<DecimalLit>().unwrap()
    }

    #[test]
    fn scale_insensitive_equality() {
        assert_eq!(lit("0.5"), lit("0.50"));
        assert_eq!(lit("1"), lit("1.000"));
    }

    #[test]
    fn ordering() {
        assert!(lit("-0.1") < lit("0.1"));
        assert!(lit("1") < lit("1.5"));
        assert!(lit("-1.5") < lit("-1"));
    }

    #[test]
    fn to_tokens_round_trips() {
        // Including an out-of-i64-range mantissa: proves we need the i128
        // constructor `from_i128_with_scale`, not `Decimal::new(i64, _)`.
        for s in ["0", "1.50", "-0.001", "100000000000000000000"] {
            let value = lit(s);
            let mantissa = value.0.mantissa();
            let scale = value.0.scale();
            let expected = quote!(
                ::rust_decimal::Decimal::from_i128_with_scale(#mantissa, #scale)
            );
            assert_eq!(value.to_token_stream().to_string(), expected.to_string());
            // And the reconstruction equals the original value.
            assert_eq!(
                rust_decimal::Decimal::from_i128_with_scale(mantissa, scale),
                value.0
            );
        }
    }

    #[test]
    fn rejects_invalid_literal() {
        assert!("1.2.3".parse::<DecimalLit>().is_err());
        // Out of Decimal's range.
        assert!("1e40".parse::<DecimalLit>().is_err());
    }
}
