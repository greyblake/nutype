use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::common::models::Attributes;
use crate::common::parse::{parse_number, ParseableAttributes};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::Token;
use syn::{
    parse::{Parse, ParseStream},
    Expr,
};

use super::{
    models::{
        IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerValidator, SpannedIntegerSanitizer,
        SpannedIntegerValidator,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(input: TokenStream) -> Result<Attributes<IntegerGuard<T>>, syn::Error>
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
    } = attrs;
    let maybe_default_value = default.map(|expr| quote!(#expr));
    let raw_guard = IntegerRawGuard {
        sanitizers,
        validators,
    };
    let guard = validate_number_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
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
        } else if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::With(quote!(#expr)),
                span: expr.span(),
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
            let expr: Expr = input.parse()?;
            Ok(SpannedIntegerSanitizer {
                item: IntegerSanitizer::With(quote!(#expr)),
                span: expr.span(),
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}
