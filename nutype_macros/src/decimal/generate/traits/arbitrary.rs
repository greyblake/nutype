use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

use crate::{
    common::models::{TypeName, Validation},
    decimal::models::{DecimalGuard, DecimalInnerType, DecimalValidator},
    utils::issue_reporter::{Issue, build_github_link_with_issue},
};

pub fn gen_impl_trait_arbitrary<T: ToTokens>(
    type_name: &TypeName,
    inner_type: &DecimalInnerType,
    guard: &DecimalGuard<T>,
) -> Result<TokenStream, syn::Error> {
    let generate_inner_value = gen_generate_valid_inner_value(inner_type, guard)?;

    let construct_value = if guard.has_validation() {
        // If by some reason we generate an invalid value, make it very easy for the user to report
        let report_issue_msg =
            build_github_link_with_issue(&Issue::ArbitraryGeneratedInvalidValue {
                inner_type: inner_type.to_string(),
            });
        let error_text =
            format!("Arbitrary generated an invalid value for {type_name}.\n\n{report_issue_msg}");
        quote!(
            Self::try_new(inner_value).expect(#error_text)
        )
    } else {
        quote!(Self::new(inner_value))
    };

    Ok(quote!(
        impl ::arbitrary::Arbitrary<'_> for #type_name {
            fn arbitrary(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<Self> {
                let inner_value: #inner_type = {
                    #generate_inner_value
                };
                Ok(#construct_value)
            }

            #[inline]
            fn size_hint(_depth: usize) -> (usize, Option<usize>) {
                let n = ::core::mem::size_of::<#inner_type>();
                (n, Some(n))
            }
        }
    ))
}

/// Generates code that produces a valid inner `Decimal` value.
fn gen_generate_valid_inner_value<T: ToTokens>(
    inner_type: &DecimalInnerType,
    guard: &DecimalGuard<T>,
) -> Result<TokenStream, syn::Error> {
    match guard {
        DecimalGuard::WithoutValidation { .. } => {
            // No validation: delegate straight to the inner type's Arbitrary impl
            // (requires the user to enable `rust_decimal/rust-fuzz`).
            Ok(quote!(u.arbitrary()?))
        }
        DecimalGuard::WithValidation {
            sanitizers,
            validation,
        } => match validation {
            Validation::Standard { validators, .. } => {
                if !sanitizers.is_empty() {
                    let msg = "It's not possible to derive `Arbitrary` for a Decimal type that has both a `with` sanitizer and validation.\nYou have to implement `Arbitrary` on your own.";
                    return Err(syn::Error::new(Span::call_site(), msg));
                }
                gen_generate_valid_inner_value_with_validators(inner_type, validators)
            }
            Validation::Custom { .. } => {
                let msg = "It's not possible to derive `Arbitrary` for a Decimal type with custom validation.\nYou have to implement `Arbitrary` on your own.";
                Err(syn::Error::new(Span::call_site(), msg))
            }
        },
    }
}

fn gen_generate_valid_inner_value_with_validators<T: ToTokens>(
    _inner_type: &DecimalInnerType,
    validators: &[DecimalValidator<T>],
) -> Result<TokenStream, syn::Error> {
    let mut lower: Option<TokenStream> = None;
    let mut upper: Option<TokenStream> = None;

    for validator in validators {
        match validator {
            DecimalValidator::Predicate(_) => {
                let msg = "It's not possible to derive `Arbitrary` for a Decimal type with a `predicate` validator.\nYou have to implement `Arbitrary` on your own.";
                return Err(syn::Error::new(Span::call_site(), msg));
            }
            DecimalValidator::Greater(_) | DecimalValidator::Less(_) => {
                let msg = "Deriving `Arbitrary` for a Decimal type currently supports inclusive bounds only (`greater_or_equal` / `less_or_equal`).\nExclusive bounds (`greater` / `less`) are not yet supported, because nudging an exact Decimal strictly past a bound is unsolved here.";
                return Err(syn::Error::new(Span::call_site(), msg));
            }
            DecimalValidator::GreaterOrEqual(value) => {
                lower = Some(value.to_token_stream());
            }
            DecimalValidator::LessOrEqual(value) => {
                upper = Some(value.to_token_stream());
            }
        }
    }

    let body = match (lower, upper) {
        (Some(lower), Some(upper)) => quote!(
            let lower: ::rust_decimal::Decimal = #lower;
            let upper: ::rust_decimal::Decimal = #upper;
            // Build a fraction in [0, 1] (u64::MAX fits comfortably in Decimal's range).
            let numerator = ::rust_decimal::Decimal::from(<u64 as ::arbitrary::Arbitrary>::arbitrary(u)?);
            let denominator = ::rust_decimal::Decimal::from(u64::MAX);
            let fraction = numerator
                .checked_div(denominator)
                .unwrap_or(::rust_decimal::Decimal::ZERO);
            // Scale into [lower, upper], using checked arithmetic and falling back
            // to `lower` (always valid) on overflow, then clamp to be safe.
            let value = upper
                .checked_sub(lower)
                .and_then(|span| fraction.checked_mul(span))
                .and_then(|delta| lower.checked_add(delta))
                .unwrap_or(lower);
            ::core::cmp::Ord::clamp(value, lower, upper)
        ),
        (Some(lower), None) => quote!(
            let lower: ::rust_decimal::Decimal = #lower;
            let base: ::rust_decimal::Decimal = u.arbitrary()?;
            lower.checked_add(base.abs()).unwrap_or(lower)
        ),
        (None, Some(upper)) => quote!(
            let upper: ::rust_decimal::Decimal = #upper;
            let base: ::rust_decimal::Decimal = u.arbitrary()?;
            upper.checked_sub(base.abs()).unwrap_or(upper)
        ),
        (None, None) => quote!(u.arbitrary()?),
    };

    Ok(body)
}
