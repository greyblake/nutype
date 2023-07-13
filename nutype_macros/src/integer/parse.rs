use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::common::parse::parse_integer;
use crate::common::{
    models::Attributes,
    parse::{is_comma, parse_nutype_attributes, parse_with_token_stream, split_and_parse},
};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
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
    <T as FromStr>::Err: Debug,
{
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_sanitize_attr)
}

fn parse_sanitize_attr<T>(tokens: Vec<TokenTree>) -> Result<SpannedIntegerSanitizer<T>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut token_iter = tokens.iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "with" => {
                // Preserve the rest as `custom_sanitizer_fn`
                let stream = parse_with_token_stream(token_iter, ident.span())?;
                let span = ident.span();
                let sanitizer = IntegerSanitizer::With(stream);
                Ok(SpannedIntegerSanitizer {
                    span,
                    item: sanitizer,
                })
            }
            unknown_sanitizer => {
                let msg = format!("Unknown sanitizer `{unknown_sanitizer}`");
                let error = syn::Error::new(ident.span(), msg);
                Err(error)
            }
        }
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax."))
    }
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
