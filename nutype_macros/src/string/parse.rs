use crate::common::models::Attributes;
use crate::common::parse::{
    is_comma, parse_nutype_attributes, parse_value_as_number, parse_with_token_stream,
    split_and_parse,
};
use crate::string::models::StringGuard;
use crate::string::models::StringRawGuard;
use crate::string::models::{StringSanitizer, StringValidator};
use crate::utils::match_feature;
use proc_macro2::{Span, TokenStream, TokenTree};

use super::models::{RegexDef, SpannedStringSanitizer, SpannedStringValidator};
use super::validate::validate_string_meta;

pub fn parse_attributes(input: TokenStream) -> Result<Attributes<StringGuard>, darling::Error> {
    let raw_attrs = parse_raw_attributes(input)?;
    let Attributes {
        new_unchecked,
        guard: raw_guard,
        maybe_default_value,
    } = raw_attrs;
    let guard = validate_string_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
    })
}

fn parse_raw_attributes(input: TokenStream) -> Result<Attributes<StringRawGuard>, darling::Error> {
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs(
    stream: TokenStream,
) -> Result<Vec<SpannedStringSanitizer>, darling::Error> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_sanitize_attr)
}

fn parse_sanitize_attr(tokens: Vec<TokenTree>) -> Result<SpannedStringSanitizer, darling::Error> {
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
                let error = syn::Error::new(ident.span(), msg).into();
                return Err(error);
            }
        };
        Ok(SpannedStringSanitizer::new(san, ident.span()))
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax.").into())
    }
}

fn parse_validate_attrs(
    stream: TokenStream,
) -> Result<Vec<SpannedStringValidator>, darling::Error> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_validate_attr)
}

// TODO: refactor
// * Avoid unnecessary allocations
// * Replace `parse_value_as_number()` with something better
fn parse_validate_attr(tokens: Vec<TokenTree>) -> Result<SpannedStringValidator, darling::Error> {
    let mut token_iter = tokens.into_iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "max_len" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = StringValidator::MaxLen(value);
                let parsed_validator = SpannedStringValidator::new(validator, ident.span());
                Ok(parsed_validator)
            }
            "min_len" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = StringValidator::MinLen(value);
                let parsed_validator = SpannedStringValidator::new(validator, ident.span());
                Ok(parsed_validator)
            }
            "not_empty" => {
                let validator = StringValidator::NotEmpty;
                let parsed_validator = SpannedStringValidator::new(validator, ident.span());
                Ok(parsed_validator)
            }
            "with" => {
                let rest_tokens: Vec<_> = token_iter.collect();
                let stream = parse_with_token_stream(rest_tokens.iter(), ident.span())?;
                let validator = StringValidator::With(stream);
                let parsed_validator = SpannedStringValidator::new(validator, ident.span());
                Ok(parsed_validator)
            }
            "regex" => {
                match_feature!("regex",
                    on => {
                        let rest_tokens: Vec<_> = token_iter.collect();
                        let stream = parse_with_token_stream(rest_tokens.iter(), ident.span())?;
                        let (regex_def, span) = parse_regex(stream, ident.span())?;
                        let validator = StringValidator::Regex(regex_def);
                        let parsed_validator = SpannedStringValidator::new(validator, span);
                        Ok(parsed_validator)
                    },
                    off => {
                        let msg = concat!(
                            "To validate string types with regex, the feature `regex` of the crate `nutype` must be enabled.\n",
                            "IMPORTANT: Make sure that your crate EXPLICITLY depends on `regex` and `lazy_static` crates.\n",
                            "And... don't forget to take care of yourself and your beloved ones. That is even more important.",
                        );
                        return Err(syn::Error::new(ident.span(), msg)).into();
                    }
                )
            }
            validator => {
                let msg = format!("Unknown validation rule `{validator}`");
                let error = syn::Error::new(ident.span(), msg).into();
                Err(error)
            }
        }
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax.").into())
    }
}

#[cfg_attr(not(feature = "regex"), allow(dead_code))]
fn parse_regex(
    stream: TokenStream,
    span: proc_macro2::Span,
) -> Result<(RegexDef, Span), darling::Error> {
    let token = stream.into_iter().next().ok_or_else(|| {
        syn::Error::new(span, "Expected a regex or an ident that refers to regex.")
    })?;
    let span = token.span();

    match token {
        TokenTree::Literal(lit) => Ok((RegexDef::StringLiteral(lit), span)),
        TokenTree::Ident(ident) => Ok((RegexDef::Ident(ident), span)),
        TokenTree::Group(..) | TokenTree::Punct(..) => Err(syn::Error::new(
            span,
            "regex must be a string or an ident that refers to Regex constant",
        )
        .into()),
    }
}
