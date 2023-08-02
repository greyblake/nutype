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
        IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerValidator, SpannedIntegerSanitizer,
        SpannedIntegerValidator,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(
    input: TokenStream,
) -> Result<Attributes<IntegerGuard<T>, SpannedDeriveTrait>, syn::Error>
where
    T: FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    let attrs: ParseableAttributes<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>> =
        syn::parse2(input)?;

    let ParseableAttributes {
        sanitizers,
        validators,
        new_unchecked,
        default,
        derive_traits,
    } = attrs;
    let raw_guard = IntegerRawGuard {
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

impl<T> Parse for SpannedIntegerValidator<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "min" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_number::<T>(input)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::Min(number),
                span,
            })
        } else if ident == "max" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_number::<T>(input)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::Max(number),
                span,
            })
        } else if ident == "predicate" {
            let _eq: Token![=] = input.parse()?;
            let (typed_custom_function, span) = parse_typed_custom_function::<&T>(input)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::Predicate(typed_custom_function),
                span,
            })
        } else {
            let msg = format!("Unknown validator `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}

impl<T> Parse for SpannedIntegerSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let (typed_custom_function, span) = parse_typed_custom_function::<T>(input)?;
            Ok(SpannedIntegerSanitizer {
                item: IntegerSanitizer::With(typed_custom_function),
                span,
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}
