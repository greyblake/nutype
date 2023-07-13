use crate::common::models::Attributes;
use crate::common::parse::{
    is_comma, parse_nutype_attributes, parse_value_as_number, parse_with_token_stream,
    split_and_parse,
};
use crate::string::models::StringGuard;
use crate::string::models::StringRawGuard;
use crate::string::models::{StringSanitizer, StringValidator};
use crate::utils::match_feature;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Token};

use super::models::{RegexDef, SpannedStringSanitizer, SpannedStringValidator};
use super::validate::validate_string_meta;

pub fn parse_attributes(input: TokenStream) -> Result<Attributes<StringGuard>, syn::Error> {
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

fn parse_raw_attributes(input: TokenStream) -> Result<Attributes<StringRawGuard>, syn::Error> {
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs(stream: TokenStream) -> Result<Vec<SpannedStringSanitizer>, syn::Error> {
    let sanitizers: ParseableSanitizers = syn::parse2(stream)?;
    Ok(sanitizers.0)
}

impl Parse for SpannedStringSanitizer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "trim" {
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::Trim,
                span: ident.span(),
            })
        } else if ident == "lowercase" {
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::Lowercase,
                span: ident.span(),
            })
        } else if ident == "uppercase" {
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::Uppercase,
                span: ident.span(),
            })
        } else if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::With(quote!(#expr)),
                span: expr.span(),
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}

#[derive(Debug)]
struct ParseableSanitizers(Vec<SpannedStringSanitizer>);

impl Parse for ParseableSanitizers {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = input.parse_terminated(SpannedStringSanitizer::parse, Token![,])?;
        let sanitizers = items.into_iter().collect();
        Ok(Self(sanitizers))
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
            "not_empty" => {
                let validator = StringValidator::NotEmpty;
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
            "regex" => {
                match_feature!("regex",
                    on => {
                        let rest_tokens: Vec<_> = token_iter.collect();
                        let stream = parse_with_token_stream(rest_tokens.iter(), ident.span())?;
                        let (regex_def, span) = parse_regex(stream, ident.span())?;
                        let validator = StringValidator::Regex(regex_def);
                        let parsed_validator = SpannedStringValidator {
                            item: validator,
                            span,
                        };
                        Ok(parsed_validator)
                    },
                    off => {
                        let msg = concat!(
                            "To validate string types with regex, the feature `regex` of the crate `nutype` must be enabled.\n",
                            "IMPORTANT: Make sure that your crate EXPLICITLY depends on `regex` and `lazy_static` crates.\n",
                            "And... don't forget to take care of yourself and your beloved ones. That is even more important.",
                        );
                        return Err(syn::Error::new(ident.span(), msg));
                    }
                )
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

#[cfg_attr(not(feature = "regex"), allow(dead_code))]
fn parse_regex(
    stream: TokenStream,
    span: proc_macro2::Span,
) -> Result<(RegexDef, Span), syn::Error> {
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
        )),
    }
}
