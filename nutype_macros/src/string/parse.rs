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

    let raw_sanitizers: Vec<ParseableStringSanitizer> = attr_args
        .iter()
        .flat_map(|arg| {
            let res = ParseableStringSanitizer::from_list(std::slice::from_ref(arg));
            errors.handle(res)
        })
        .collect();

    let raw_sanitizers = errors.finish_with(raw_sanitizers)?;

    let sanitizers: Vec<SpannedStringSanitizer> = raw_sanitizers
        .into_iter()
        .flat_map(ParseableStringSanitizer::into_spanned_string_sanitizer)
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

// Generates enums that can be used to parse attributes with darling.
//
// Example:
//
//     define_parseable_enum! {
//         parseable_name: ParseableStringSanitizer,
//         raw_name: RawStringSanitizer,
//         variants: {
//             Trim: bool,
//             Lowercase: bool,
//         }
//     }
//
// Generates the following:
//
//     #[derive(Debug, FromMeta)]
//     enum ParseableStringSanitizer {
//         Trim(SpannedValue<bool>),
//         Lowercase(SpannedValue<bool>),
//     }
//
//     #[derive(Debug, Clone)]
//     enum RawStringSanitizer {
//         Trim(bool),
//         Lowercase(bool),
//     }
//
//     impl ParseableStringSanitizer {
//         fn into_spanned_raw(self) -> SpannedValue<RawStringSanitizer> {
//             use ParseableStringSanitizer::*;
//
//             match self {
//                 Trim(sv) => {
//                     let raw = RawStringSanitizer::Trim(sv.as_ref().to_owned());
//                     SpannedValue::new(raw, sv.span())
//                 }
//                 Lowercase(sv) => {
//                     let raw = RawStringSanitizer::Lowercase(sv.as_ref().to_owned());
//                     SpannedValue::new(raw, sv.span())
//                 }
//             }
//         }
//     }
//
macro_rules! define_parseable_enum {
    (
        parseable_name: $parseable_name:ident,
        raw_name: $raw_name:ident,
        variants: { $($vname:ident: $vtype:ty),+, }
    ) => {

        #[derive(Debug, FromMeta)]
        enum $parseable_name {
            $(
                $vname(SpannedValue<$vtype>)
            ),+
        }

        #[derive(Debug, Clone)]
        enum $raw_name {
            $(
                $vname($vtype)
            ),+
        }

        impl $parseable_name {
            fn into_spanned_raw(self) -> SpannedValue<$raw_name> {
                match self {
                    $(
                        $parseable_name::$vname(sv) => {
                            let raw = $raw_name::$vname(sv.as_ref().to_owned());
                            SpannedValue::new(raw, sv.span())
                        }
                    ),+
                }
            }
        }
    };
}

define_parseable_enum! {
    parseable_name: ParseableStringSanitizer,
    raw_name: RawStringSanitizer,
    variants: {
        Trim: bool,
        Lowercase: bool,
        Uppercase: bool,
        With: Expr,
    }
}

impl ParseableStringSanitizer {
    fn into_spanned_string_sanitizer(self) -> Option<SpannedStringSanitizer> {
        use RawStringSanitizer::*;

        let spanned_raw = self.into_spanned_raw();
        let span = spanned_raw.span();
        let raw = spanned_raw.as_ref();

        let maybe_sanitizer = match raw {
            Trim(true) => Some(StringSanitizer::Trim),
            Trim(false) => None,
            Lowercase(true) => Some(StringSanitizer::Lowercase),
            Lowercase(false) => None,
            Uppercase(true) => Some(StringSanitizer::Uppercase),
            Uppercase(false) => None,
            With(expr) => Some(StringSanitizer::With(quote::quote!(#expr))),
        };

        maybe_sanitizer.map(|san| SpannedValue::new(san, span))
    }
}
