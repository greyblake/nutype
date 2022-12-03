use std::fmt::Debug;
use std::str::FromStr;

use crate::common::parse::{
    is_comma, parse_nutype_attributes, parse_value_as_number, parse_with_token_stream,
    split_and_parse, try_unwrap_group,
};
use proc_macro2::{Span, TokenStream, TokenTree};

use super::{
    models::{
        FloatSanitizer, FloatValidator, NewtypeFloatMeta, RawNewtypeFloatMeta,
        SpannedFloatSanitizer, SpannedFloatValidator,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(input: TokenStream) -> Result<NewtypeFloatMeta<T>, syn::Error>
where
    T: FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug,
{
    parse_raw_attributes(input).and_then(validate_number_meta)
}

fn parse_raw_attributes<T>(input: TokenStream) -> Result<RawNewtypeFloatMeta<T>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs<T>(stream: TokenStream) -> Result<Vec<SpannedFloatSanitizer<T>>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_sanitize_attr)
}

fn parse_sanitize_attr<T>(tokens: Vec<TokenTree>) -> Result<SpannedFloatSanitizer<T>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut token_iter = tokens.iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "clamp" => {
                let t = token_iter.next().expect("clamp() cannot be empty");
                let span = ident.span();
                let group = try_unwrap_group(t.clone())?;
                let list: Vec<T> = parse_list_of_numbers(group.stream());
                if list.len() != 2 {
                    let msg = "Invalid parameters for clamp()";
                    let error = syn::Error::new(span, msg);
                    return Err(error);
                }

                let mut iter = list.into_iter();
                let min = iter.next().unwrap();
                let max = iter.next().unwrap();
                let sanitizer = FloatSanitizer::Clamp { min, max };
                Ok(SpannedFloatSanitizer {
                    span,
                    item: sanitizer,
                })
            }
            "with" => {
                // Preserve the rest as `custom_sanitizer_fn`
                let stream = parse_with_token_stream(token_iter, ident.span())?;
                let span = ident.span();
                let sanitizer = FloatSanitizer::With(stream);
                Ok(SpannedFloatSanitizer {
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

fn parse_validate_attrs<T>(stream: TokenStream) -> Result<Vec<SpannedFloatValidator<T>>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_validate_attr)
}

fn parse_validate_attr<T>(tokens: Vec<TokenTree>) -> Result<SpannedFloatValidator<T>, syn::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut token_iter = tokens.into_iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "min" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = FloatValidator::Min(value);
                let parsed_validator = SpannedFloatValidator {
                    span: ident.span(),
                    item: validator,
                };
                Ok(parsed_validator)
            }
            "max" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = FloatValidator::Max(value);
                let parsed_validator = SpannedFloatValidator {
                    span: ident.span(),
                    item: validator,
                };
                Ok(parsed_validator)
            }
            "with" => {
                let rest_tokens: Vec<_> = token_iter.collect();
                let stream = parse_with_token_stream(rest_tokens.iter(), ident.span())?;
                let span = ident.span();
                let validator = FloatValidator::With(stream);
                Ok(SpannedFloatValidator {
                    span,
                    item: validator,
                })
            }
            unknown_validator => {
                let msg = format!("Unknown validation rule `{unknown_validator}`");
                let error = syn::Error::new(ident.span(), msg);
                Err(error)
            }
        }
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax."))
    }
}

// TODO: Refactor
fn parse_list_of_numbers<T>(stream: TokenStream) -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut output: Vec<T> = Vec::new();
    let mut cur = String::new();

    for token in stream.into_iter() {
        let t = token.to_string();
        if t == "," {
            if !cur.is_empty() {
                // TODO: result an Result and error
                let val: T = cur.parse().unwrap();
                output.push(val);
                cur = String::new();
            }
        } else {
            cur.push_str(&t);
        }
    }
    if !cur.is_empty() {
        // TODO: result an Result and error
        let val: T = cur.parse().unwrap();
        output.push(val);
    }

    output
}