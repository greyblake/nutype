use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::common::parse::parse_number;
use crate::common::{models::Attributes, parse::ParseableAttributes};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Expr;
use syn::Token;

use super::{
    models::{
        FloatGuard, FloatRawGuard, FloatSanitizer, FloatValidator, SpannedFloatSanitizer,
        SpannedFloatValidator,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(input: TokenStream) -> Result<Attributes<FloatGuard<T>>, syn::Error>
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
    } = attrs;
    let maybe_default_value = default.map(|expr| quote!(#expr));
    let raw_guard = FloatRawGuard {
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
        } else if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            Ok(SpannedFloatValidator {
                item: FloatValidator::With(quote!(#expr)),
                span: expr.span(),
            })
        } else if ident == "finite" {
            let validator = FloatValidator::Finite;
            Ok(SpannedFloatValidator {
                item: validator,
                span: ident.span(),
            })
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
            let expr: Expr = input.parse()?;
            Ok(SpannedFloatSanitizer {
                item: FloatSanitizer::With(quote!(#expr)),
                span: expr.span(),
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}
