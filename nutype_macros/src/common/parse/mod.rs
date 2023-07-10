pub mod meta;

use std::{any::type_name, fmt::Debug, str::FromStr};

use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};

use crate::{
    common::models::{DeriveTrait, NormalDeriveTrait, RawGuard, SpannedDeriveTrait},
    utils::match_feature,
};

use super::models::{Attributes, NewUnchecked};

/// ## Example
/// Input (token stream):
///     = 123
/// Output (parsed value):
///    123
pub fn parse_value_as_number<T, ITER>(mut iter: ITER) -> Result<(T, ITER), darling::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
    ITER: Iterator<Item = TokenTree>,
{
    let token_eq = iter.next().expect("Expected token `=`");
    assert_eq!(token_eq.to_string(), "=", "Expected token `=`");

    let (num_str, span) = read_number(&mut iter)?;

    let value: T = sanitize_number(&num_str).parse::<T>().map_err(|_err| {
        let msg = format!("Expected {}, got `{}`", type_name::<T>(), num_str);
        syn::Error::new(span, msg)
    })?;

    Ok((value, iter))
}

fn read_number<ITER>(iter: &mut ITER) -> Result<(String, Span), darling::Error>
where
    ITER: Iterator<Item = TokenTree>,
{
    let mut output = String::with_capacity(16);
    let mut token_value = iter.next().expect("Expected number");
    let span = token_value.span();
    let mut t = token_value.to_string();

    // If it starts with `-` (negative number), add it to output and parse the next token.
    if t == "-" {
        output.push_str(&t);
        token_value = iter.next().expect("Expected number");
        t = token_value.to_string();
    }

    output.push_str(&t);
    Ok((output, span))
}

fn sanitize_number(val: &str) -> String {
    val.replace('_', "")
}

pub fn try_unwrap_ident(token: TokenTree) -> Result<Ident, darling::Error> {
    match token {
        TokenTree::Ident(ident) => Ok(ident),
        _ => {
            let error = syn::Error::new(token.span(), "#[nutype] expected ident").into();
            Err(error)
        }
    }
}

pub fn try_unwrap_group(token: TokenTree) -> Result<Group, darling::Error> {
    match token {
        TokenTree::Group(group) => Ok(group),
        _ => {
            let error = syn::Error::new(token.span(), "#[nutype] expected group").into();
            Err(error)
        }
    }
}

pub fn parse_nutype_attributes<S, V>(
    parse_sanitize_attrs: impl Fn(TokenStream) -> Result<Vec<S>, darling::Error>,
    parse_validate_attrs: impl Fn(TokenStream) -> Result<Vec<V>, darling::Error>,
) -> impl FnOnce(TokenStream) -> Result<Attributes<RawGuard<S, V>>, darling::Error> {
    move |input: TokenStream| {
        let mut raw_guard = RawGuard {
            sanitizers: vec![],
            validators: vec![],
        };

        // The variable `new_unchecked` is needed to be mutable for the case when `new_unchecked`
        // feature flag is enabled.
        #[allow(unused_mut)]
        let mut new_unchecked = NewUnchecked::Off;

        // Value which is used to derive Default trait
        let mut maybe_default_value: Option<TokenStream> = None;

        let mut iter = input.into_iter();

        loop {
            let token = match iter.next() {
                Some(t) => t,
                None => {
                    let attributes = Attributes {
                        guard: raw_guard,
                        new_unchecked,
                        maybe_default_value,
                    };
                    return Ok(attributes);
                }
            };

            let ident = try_unwrap_ident(token)?;

            match ident.to_string().as_ref() {
                "sanitize" => {
                    let token = iter.next().ok_or_else(|| {
                        let msg = "`sanitize` must be used with parenthesis.\nFor example:\n\n    sanitize(trim)\n\n";
                        syn::Error::new(ident.span(), msg)
                    })?;
                    let group = try_unwrap_group(token)?;
                    let sanitize_stream = group.stream();
                    raw_guard.sanitizers = parse_sanitize_attrs(sanitize_stream)?;
                }
                "validate" => {
                    let token = iter.next().ok_or_else(|| {
                        let msg = "`validate` must be used with parenthesis.\nFor example:\n\n    validate(max = 99)\n\n";
                        syn::Error::new(ident.span(), msg)
                    })?;
                    let group = try_unwrap_group(token)?;
                    let validate_stream = group.stream();
                    raw_guard.validators = parse_validate_attrs(validate_stream)?;
                }
                "default" => {
                    {
                        // Take `=` sign
                        if let Some(eq_t) = iter.next() {
                            if !is_eq(&eq_t) {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "Invalid syntax for `default`. Expected `=`, got `{eq_t}`",
                                )
                                .into());
                            }
                        } else {
                            return Err(syn::Error::new(
                                ident.span(),
                                "Invalid syntax for `default`. Missing `=`",
                            )
                            .into());
                        }
                    }
                    // TODO: parse it properly till some delimeter?
                    let default_value = iter
                        .next()
                        .ok_or_else(|| syn::Error::new(ident.span(), "Missing default value"))?;
                    maybe_default_value = Some(TokenStream::from(default_value));
                }
                "new_unchecked" => {
                    match_feature!("new_unchecked",
                        // The feature is not enabled, so we return an error
                        on => {
                            // Try to look forward and return a good helpful error if `new_unchecked` is
                            // used incorrectly.
                            // Correct:
                            //    new_unchecked
                            // Incorrect:
                            //    new_unchecked()
                            //    new_unchecked(foo = 13)
                            let maybe_next_token = iter.clone().next();
                            match maybe_next_token.map(try_unwrap_ident) {
                                None | Some(Ok(_)) => {
                                    new_unchecked = NewUnchecked::On;
                                }
                                Some(Err(err)) => {
                                    let msg = "new_unchecked does not support any options";
                                    return Err(syn::Error::new(err.span(), msg).into());
                                }
                            }
                        },

                        // The feature `new_unchecked` is enabled
                        off => {
                            let msg = "To generate ::new_unchecked() function, the feature `new_unchecked` of crate `nutype` needs to be enabled.\nBut you ought to know: generally, THIS IS A BAD IDEA.\nUse it only when you really need it.";
                            return Err(syn::Error::new(ident.span(), msg).into());
                        }
                    )
                }
                unknown => {
                    let msg = format!("Unknown #[nutype] option: `{unknown}`");
                    let error = syn::Error::new(ident.span(), msg).into();
                    return Err(error);
                }
            }
        }
    }
}

pub fn split_and_parse<SEP, PRS, OUT>(
    tokens: Vec<TokenTree>,
    is_separator: SEP,
    parse: PRS,
) -> Result<Vec<OUT>, darling::Error>
where
    SEP: Fn(&TokenTree) -> bool,
    PRS: Fn(Vec<TokenTree>) -> Result<OUT, darling::Error>,
{
    tokens
        .split(is_separator)
        .filter(|subtokens| !subtokens.is_empty())
        .map(|subtokens| parse(subtokens.to_owned()))
        .collect()
}

pub fn is_comma(token: &TokenTree) -> bool {
    match token {
        TokenTree::Punct(punct) => punct.as_char() == ',',
        _ => false,
    }
}

pub fn is_eq(token: &TokenTree) -> bool {
    match token {
        TokenTree::Punct(punct) => punct.as_char() == '=',
        _ => false,
    }
}

// Context:
//   with = |s: String| s.uppercase()
// Input:
//   = |s: String| s.to_uppercase()
// Output
//   |s: String| s.to_uppercase()
pub fn parse_with_token_stream<'a>(
    mut token_iter: impl Iterator<Item = &'a TokenTree>,
    with_span: Span,
) -> Result<TokenStream, darling::Error> {
    {
        // Take `=` sign
        if let Some(eq_t) = token_iter.next() {
            if !is_eq(eq_t) {
                let span = with_span;
                return Err(syn::Error::new(
                    span,
                    "Invalid syntax for `with`. Expected `=`, got `{eq_t}`",
                )
                .into());
            }
        } else {
            return Err(
                syn::Error::new(with_span, "Invalid syntax for `with`. Missing `=`").into(),
            );
        }
    }

    // Return the rest as TokenStream
    let rest = TokenStream::from_iter(token_iter.cloned());
    Ok(rest)
}

pub fn is_doc_attribute(attribute: &syn::Attribute) -> bool {
    match attribute.path().segments.first() {
        Some(path_segment) => path_segment.ident == "doc",
        None => false,
    }
}

pub fn is_derive_attribute(attribute: &syn::Attribute) -> bool {
    match attribute.path().segments.first() {
        Some(path_segment) => path_segment.ident == "derive",
        None => false,
    }
}

pub fn parse_derive_traits(
    attributes: &[syn::Attribute],
) -> Result<Vec<SpannedDeriveTrait>, darling::Error> {
    let traits: Vec<Vec<SpannedDeriveTrait>> = attributes
        .iter()
        .filter(|a| is_derive_attribute(a))
        .map(parse_derive_attr)
        .collect::<Result<_, darling::Error>>()?;
    Ok(traits.into_iter().flatten().collect())
}

fn parse_derive_attr(attr: &syn::Attribute) -> Result<Vec<SpannedDeriveTrait>, darling::Error> {
    let meta = &attr.meta;
    match meta {
        syn::Meta::Path(path) => {
            let msg = format!("Expected #[derive(...)], got: {path:?}");
            Err(syn::Error::new(Span::call_site(), msg).into())
        }
        syn::Meta::NameValue(name_value) => {
            let msg = format!("Expected #[derive(...)], got: {name_value:?}");
            Err(syn::Error::new(Span::call_site(), msg).into())
        }
        syn::Meta::List(list) => {
            let derive_traits: Vec<SpannedDeriveTrait> = list
                .tokens
                .clone()
                .into_iter()
                .map(parse_token_into_derive_trait)
                .collect::<Result<Vec<Option<SpannedDeriveTrait>>, darling::Error>>()?
                .into_iter()
                .flatten()
                .collect();
            Ok(derive_traits)
        }
    }
}

fn parse_token_into_derive_trait(
    token: TokenTree,
) -> Result<Option<SpannedDeriveTrait>, darling::Error> {
    match token {
        TokenTree::Ident(ident) => {
            let derive_trait = parse_ident_into_derive_trait(ident)?;
            Ok(Some(derive_trait))
        }
        TokenTree::Punct(ref punct) => match punct.as_char() {
            ',' => Ok(None),
            '*' => {
                let spanned_trait = SpannedDeriveTrait::new(DeriveTrait::Asterisk, token.span());
                Ok(Some(spanned_trait))
            }
            _ => Err(syn::Error::new(token.span(), format!("Unexpected `{token}`")).into()),
        },
        _ => Err(syn::Error::new(token.span(), format!("Unexpected `{token}`")).into()),
    }
}

fn parse_ident_into_derive_trait(ident: Ident) -> Result<SpannedDeriveTrait, darling::Error> {
    let normal_derive_trait = match ident.to_string().as_ref() {
        "Debug" => NormalDeriveTrait::Debug,
        "Display" => NormalDeriveTrait::Display,
        "Clone" => NormalDeriveTrait::Clone,
        "Copy" => NormalDeriveTrait::Copy,
        "PartialEq" => NormalDeriveTrait::PartialEq,
        "Eq" => NormalDeriveTrait::Eq,
        "PartialOrd" => NormalDeriveTrait::PartialOrd,
        "Ord" => NormalDeriveTrait::Ord,
        "FromStr" => NormalDeriveTrait::FromStr,
        "AsRef" => NormalDeriveTrait::AsRef,
        "Deref" => NormalDeriveTrait::Deref,
        "TryFrom" => NormalDeriveTrait::TryFrom,
        "From" => NormalDeriveTrait::From,
        "Into" => NormalDeriveTrait::Into,
        "Hash" => NormalDeriveTrait::Hash,
        "Borrow" => NormalDeriveTrait::Borrow,
        "Default" => NormalDeriveTrait::Default,
        "Serialize" => {
            match_feature!("serde",
                on => NormalDeriveTrait::SerdeSerialize,
                off => {
                    return Err(syn::Error::new(ident.span(), "To derive Serialize, the feature `serde` of the crate `nutype` needs to be enabled.").into());
                },
            )
        }
        "Deserialize" => {
            match_feature!("serde",
                on => NormalDeriveTrait::SerdeDeserialize,
                off => {
                    return Err(syn::Error::new(ident.span(), "To derive Deserialize, the feature `serde` of the crate `nutype` needs to be enabled.").into());
                },
            )
        }
        "JsonSchema" => {
            match_feature!("schemars08",
                on => NormalDeriveTrait::SchemarsJsonSchema,
                off => {
                    return Err(syn::Error::new(ident.span(), "To derive JsonSchema, the feature `schemars08` of the crate `nutype` needs to be enabled.").into());
                }
            )
        }
        _ => {
            return Err(syn::Error::new(
                ident.span(),
                format!("unsupported trait derive: {ident}"),
            )
            .into());
        }
    };
    let derive_trait = DeriveTrait::Normal(normal_derive_trait);
    let spanned_trait = SpannedDeriveTrait::new(derive_trait, ident.span());
    Ok(spanned_trait)
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
pub(crate) use define_parseable_enum;
