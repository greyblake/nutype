use crate::common::models::Attributes;
use crate::common::parse::{
    is_comma, parse_nutype_attributes, parse_value_as_number, parse_with_token_stream,
    split_and_parse,
};
use crate::string::models::StringGuard;
use crate::string::models::StringRawGuard;
use crate::string::models::{StringSanitizer, StringValidator};
use crate::utils::match_feature;
use darling::export::NestedMeta;
use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::Expr;

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
    let attr_args = NestedMeta::parse_meta_list(stream)?;

    let mut errors = darling::Error::accumulator();

    let raw_sanitizers: Vec<RawStringSanitizer> = attr_args
        .iter()
        .flat_map(|arg| {
            let res = RawStringSanitizer::from_list(std::slice::from_ref(arg));
            errors.handle(res)
        })
        .collect();

    let raw_sanitizers = errors.finish_with(raw_sanitizers)?;


    let sanitizers: Vec<SpannedStringSanitizer> = raw_sanitizers.into_iter()
        .flat_map(RawStringSanitizer::into_spanned_string_sanitizer)
        .collect();

    Ok(sanitizers)
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

#[derive(Debug, FromMeta)]
enum RawStringSanitizer {
    Trim(SpannedValue<bool>),
    Lowercase(SpannedValue<bool>),
    Uppercase(SpannedValue<bool>),
    With(SpannedValue<Expr>),
}

impl RawStringSanitizer {
    fn into_spanned_string_sanitizer(self) -> Option<SpannedStringSanitizer> {
        match self {
            RawStringSanitizer::Trim(flag) if *flag => Some(SpannedStringSanitizer::new(StringSanitizer::Trim, flag.span())),
            RawStringSanitizer::Trim(_) => None,
            RawStringSanitizer::Lowercase(flag) if *flag => Some(SpannedStringSanitizer::new(StringSanitizer::Lowercase, flag.span())),
            RawStringSanitizer::Lowercase(_) => None,
            RawStringSanitizer::Uppercase(flag) if *flag => Some(SpannedStringSanitizer::new(StringSanitizer::Uppercase, flag.span())),
            RawStringSanitizer::Uppercase(_) => None,
            RawStringSanitizer::With(val) => {
                let expr = val.as_ref();
                let with_sanitizer = StringSanitizer::With(quote::quote!(#expr));
                Some(SpannedStringSanitizer::new(with_sanitizer, val.span()))
            }
        }
    }
}
