use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::common::{
    models::{Attributes, SpannedDeriveTrait},
    parse::{parse_number, parse_typed_custom_function, ParseableAttributes},
};
use proc_macro2::{Ident, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use super::{
    models::{
        FloatGuard, FloatRawGuard, FloatSanitizer, FloatValidator, SpannedFloatSanitizer,
        SpannedFloatValidator,
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
        let ident: Ident = input.parse()?;

        if ident == "min" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_number::<T>(input)?;
            Ok(SpannedFloatValidator {
                item: FloatValidator::Min(number as T),
                span,
            })
        } else if ident == "max" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_number::<T>(input)?;
            Ok(SpannedFloatValidator {
                item: FloatValidator::Max(number as T),
                span,
            })
        } else if ident == "predicate" {
            let _eq: Token![=] = input.parse()?;
            let (typed_custom_function, span) = parse_typed_custom_function::<&T>(input)?;
            Ok(SpannedFloatValidator {
                item: FloatValidator::Predicate(typed_custom_function),
                span,
            })
        } else if ident == "finite" {
            let validator = FloatValidator::Finite;
            Ok(SpannedFloatValidator {
                item: validator,
                span: ident.span(),
            })
        } else if ident == "with" {
            let msg = "Deprecated validator `with`. It was renamed to `predicate`";
            Err(syn::Error::new(ident.span(), msg))
        } else {
            let msg = format!("Unknown validator `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}

impl<T> Parse for SpannedFloatSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let (typed_custom_function, span) = parse_typed_custom_function::<T>(input)?;
            Ok(SpannedFloatSanitizer {
                item: FloatSanitizer::With(typed_custom_function),
                span,
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}
