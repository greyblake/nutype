use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::common::{
    models::{Attributes, SpannedDeriveTrait},
    parse::{
        parse_number, parse_sanitizer_kind, parse_typed_custom_function, parse_validator_kind,
        ParseableAttributes,
    },
};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use super::{
    models::{
        FloatGuard, FloatRawGuard, FloatSanitizer, FloatSanitizerKind, FloatValidator,
        FloatValidatorKind, SpannedFloatSanitizer, SpannedFloatValidator,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(
    input: TokenStream,
) -> Result<Attributes<FloatGuard<T>, SpannedDeriveTrait>, syn::Error>
where
    T: FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    let attrs: ParseableAttributes<SpannedFloatSanitizer<T>, SpannedFloatValidator<T>> =
        syn::parse2(input)?;

    let ParseableAttributes {
        sanitizers,
        validators,
        new_unchecked,
        default,
        derive_traits,
    } = attrs;
    let raw_guard = FloatRawGuard {
        sanitizers,
        validators,
    };
    let guard = validate_number_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        default,
        derive_traits,
    })
}

impl<T> Parse for SpannedFloatValidator<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, ident) = parse_validator_kind(input)?;

        match kind {
            FloatValidatorKind::Greater => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number::<T>(input)?;
                Ok(SpannedFloatValidator {
                    item: FloatValidator::Greater(number),
                    span,
                })
            }
            FloatValidatorKind::GreaterOrEqual => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number::<T>(input)?;
                Ok(SpannedFloatValidator {
                    item: FloatValidator::GreaterOrEqual(number),
                    span,
                })
            }
            FloatValidatorKind::Less => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number::<T>(input)?;
                Ok(SpannedFloatValidator {
                    item: FloatValidator::Less(number),
                    span,
                })
            }
            FloatValidatorKind::LessOrEqual => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number::<T>(input)?;
                Ok(SpannedFloatValidator {
                    item: FloatValidator::LessOrEqual(number),
                    span,
                })
            }
            FloatValidatorKind::Predicate => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function::<&T>(input)?;
                Ok(SpannedFloatValidator {
                    item: FloatValidator::Predicate(typed_custom_function),
                    span,
                })
            }
            FloatValidatorKind::Finite => {
                let validator = FloatValidator::Finite;
                Ok(SpannedFloatValidator {
                    item: validator,
                    span: ident.span(),
                })
            }
        }
    }
}

impl<T> Parse for SpannedFloatSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, ident) = parse_sanitizer_kind(input)?;

        match kind {
            FloatSanitizerKind::With => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function::<T>(input)?;
                Ok(SpannedFloatSanitizer {
                    item: FloatSanitizer::With(typed_custom_function),
                    span,
                })
            }
            FloatSanitizerKind::_Phantom => {
                let msg = format!("Unknown validator `{ident}`");
                Err(syn::Error::new(ident.span(), msg))
            }
        }
    }
}
