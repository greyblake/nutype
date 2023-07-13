use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::common::parse::parse_integer;
use crate::common::{models::Attributes, parse::parse_nutype_attributes};
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
    let raw_attrs = parse_raw_attributes(input)?;
    let Attributes {
        new_unchecked,
        guard: raw_guard,
        maybe_default_value,
    } = raw_attrs;
    let guard = validate_number_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
    })
}

fn parse_raw_attributes<T>(input: TokenStream) -> Result<Attributes<IntegerRawGuard<T>>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug + std::fmt::Display,
{
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs<T>(
    stream: TokenStream,
) -> Result<Vec<SpannedIntegerSanitizer<T>>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug + Display,
{
    let sanitizers: Sanitizers<T> = syn::parse2(stream)?;
    Ok(sanitizers.0)
}

fn parse_validate_attrs<T>(
    stream: TokenStream,
) -> Result<Vec<SpannedIntegerValidator<T>>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug + std::fmt::Display,
{
    let validators: Validators<T> = syn::parse2(stream)?;
    Ok(validators.0)
}

impl<T> Parse for SpannedIntegerValidator<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "min" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_integer::<T>(input)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::Min(number),
                span,
            })
        } else if ident == "max" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_integer::<T>(input)?;
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

struct Validators<T>(Vec<SpannedIntegerValidator<T>>);

impl<T> Parse for Validators<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = input.parse_terminated(SpannedIntegerValidator::parse, Token![,])?;
        let validators: Vec<SpannedIntegerValidator<T>> = items.into_iter().collect();
        Ok(Self(validators))
    }
}

impl<T> Parse for SpannedIntegerSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
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

struct Sanitizers<T>(Vec<SpannedIntegerSanitizer<T>>);

impl<T> Parse for Sanitizers<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = input.parse_terminated(SpannedIntegerSanitizer::parse, Token![,])?;
        let sanitizers: Vec<SpannedIntegerSanitizer<T>> = items.into_iter().collect();
        Ok(Self(sanitizers))
    }
}
