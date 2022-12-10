use crate::common::parse::{
    is_comma, parse_nutype_attributes, parse_value_as_number, parse_with_token_stream,
    split_and_parse,
};
use crate::models::{StringSanitizer, StringValidator};
use crate::string::models::StringGuard;
use crate::string::models::StringRawGuard;
use proc_macro2::{Span, TokenStream, TokenTree};

use super::models::{SpannedStringSanitizer, SpannedStringValidator};
use super::validate::validate_string_meta;

pub fn parse_attributes(input: TokenStream) -> Result<StringGuard, syn::Error> {
    parse_raw_attributes(input).and_then(validate_string_meta)
}

fn parse_raw_attributes(input: TokenStream) -> Result<StringRawGuard, syn::Error> {
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs(stream: TokenStream) -> Result<Vec<SpannedStringSanitizer>, syn::Error> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_sanitize_attr)
}

fn parse_sanitize_attr(tokens: Vec<TokenTree>) -> Result<SpannedStringSanitizer, syn::Error> {
    let mut token_iter = tokens.iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        let san = match ident.to_string().as_ref() {
            "trim" => StringSanitizer::Trim,
            "lowercase" => StringSanitizer::Lowercase,
            "uppercase" => StringSanitizer::Uppercase,
            "with" => {
                // Preserve the rest as `custom_sanitizer_fn`
                let stream = parse_with_token_stream(token_iter, ident.span())?;
                StringSanitizer::With(stream)
            }
            unknown_sanitizer => {
                let msg = format!("Unknown sanitizer `{unknown_sanitizer}`");
                let error = syn::Error::new(ident.span(), msg);
                return Err(error);
            }
        };
        Ok(SpannedStringSanitizer {
            span: ident.span(),
            item: san,
        })
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax."))
    }
}

fn parse_validate_attrs(stream: TokenStream) -> Result<Vec<SpannedStringValidator>, syn::Error> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_validate_attr)
}

// TODO: refactor
// * Avoid unnecessary allocations
// * Replace `parse_value_as_number()` with something better
fn parse_validate_attr(tokens: Vec<TokenTree>) -> Result<SpannedStringValidator, syn::Error> {
    let mut token_iter = tokens.into_iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "max_len" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = StringValidator::MaxLen(value);
                let parsed_validator = SpannedStringValidator {
                    item: validator,
                    span: ident.span(),
                };
                Ok(parsed_validator)
            }
            "min_len" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = StringValidator::MinLen(value);
                let parsed_validator = SpannedStringValidator {
                    item: validator,
                    span: ident.span(),
                };
                Ok(parsed_validator)
            }
            "present" => {
                let validator = StringValidator::Present;
                let parsed_validator = SpannedStringValidator {
                    item: validator,
                    span: ident.span(),
                };
                Ok(parsed_validator)
            }
            "with" => {
                let rest_tokens: Vec<_> = token_iter.collect();
                let stream = parse_with_token_stream(rest_tokens.iter(), ident.span())?;
                let validator = StringValidator::With(stream);
                let parsed_validator = SpannedStringValidator {
                    item: validator,
                    span: ident.span(),
                };
                Ok(parsed_validator)
            }
            validator => {
                let msg = format!("Unknown validation rule `{validator}`");
                let error = syn::Error::new(ident.span(), msg);
                Err(error)
            }
        }
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax."))
    }
}
