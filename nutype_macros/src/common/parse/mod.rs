pub mod meta;

use std::{any::type_name, fmt::Debug, str::FromStr};

use proc_macro2::{Ident, Span, TokenTree};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Expr, Lit, Token,
};

use crate::{
    common::models::{DeriveTrait, SpannedDeriveTrait},
    utils::match_feature,
};

use super::models::NewUnchecked;

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
) -> Result<Vec<SpannedDeriveTrait>, syn::Error> {
    let traits: Vec<Vec<SpannedDeriveTrait>> = attributes
        .iter()
        .filter(|a| is_derive_attribute(a))
        .map(parse_derive_attr)
        .collect::<Result<_, syn::Error>>()?;
    Ok(traits.into_iter().flatten().collect())
}

fn parse_derive_attr(attr: &syn::Attribute) -> Result<Vec<SpannedDeriveTrait>, syn::Error> {
    let meta = &attr.meta;
    match meta {
        syn::Meta::Path(path) => {
            let msg = format!("Expected #[derive(...)], got: {path:?}");
            Err(syn::Error::new(Span::call_site(), msg))
        }
        syn::Meta::NameValue(name_value) => {
            let msg = format!("Expected #[derive(...)], got: {name_value:?}");
            Err(syn::Error::new(Span::call_site(), msg))
        }
        syn::Meta::List(list) => {
            let derive_traits: Vec<SpannedDeriveTrait> = list
                .tokens
                .clone()
                .into_iter()
                .map(parse_token_into_derive_trait)
                .collect::<Result<Vec<Option<SpannedDeriveTrait>>, syn::Error>>()?
                .into_iter()
                .flatten()
                .collect();
            Ok(derive_traits)
        }
    }
}

fn parse_token_into_derive_trait(
    token: TokenTree,
) -> Result<Option<SpannedDeriveTrait>, syn::Error> {
    match token {
        TokenTree::Ident(ident) => {
            let derive_trait = parse_ident_into_derive_trait(ident)?;
            Ok(Some(derive_trait))
        }
        TokenTree::Punct(ref punct) => match punct.as_char() {
            ',' => Ok(None),
            '*' => Err(syn::Error::new(
                token.span(),
                "Asterisk derive is not longer supported",
            )),
            _ => Err(syn::Error::new(
                token.span(),
                format!("Unexpected `{token}`"),
            )),
        },
        _ => Err(syn::Error::new(
            token.span(),
            format!("Unexpected `{token}`"),
        )),
    }
}

fn parse_ident_into_derive_trait(ident: Ident) -> Result<SpannedDeriveTrait, syn::Error> {
    let derive_trait = match ident.to_string().as_ref() {
        "Debug" => DeriveTrait::Debug,
        "Display" => DeriveTrait::Display,
        "Clone" => DeriveTrait::Clone,
        "Copy" => DeriveTrait::Copy,
        "PartialEq" => DeriveTrait::PartialEq,
        "Eq" => DeriveTrait::Eq,
        "PartialOrd" => DeriveTrait::PartialOrd,
        "Ord" => DeriveTrait::Ord,
        "FromStr" => DeriveTrait::FromStr,
        "AsRef" => DeriveTrait::AsRef,
        "Deref" => DeriveTrait::Deref,
        "TryFrom" => DeriveTrait::TryFrom,
        "From" => DeriveTrait::From,
        "Into" => DeriveTrait::Into,
        "Hash" => DeriveTrait::Hash,
        "Borrow" => DeriveTrait::Borrow,
        "Default" => DeriveTrait::Default,
        "Serialize" => {
            match_feature!("serde",
                on => DeriveTrait::SerdeSerialize,
                off => {
                    return Err(syn::Error::new(ident.span(), "To derive Serialize, the feature `serde` of the crate `nutype` needs to be enabled."));
                },
            )
        }
        "Deserialize" => {
            match_feature!("serde",
                on => DeriveTrait::SerdeDeserialize,
                off => {
                    return Err(syn::Error::new(ident.span(), "To derive Deserialize, the feature `serde` of the crate `nutype` needs to be enabled."));
                },
            )
        }
        "JsonSchema" => {
            match_feature!("schemars08",
                on => DeriveTrait::SchemarsJsonSchema,
                off => {
                    return Err(syn::Error::new(ident.span(), "To derive JsonSchema, the feature `schemars08` of the crate `nutype` needs to be enabled."));
                }
            )
        }
        _ => {
            return Err(syn::Error::new(
                ident.span(),
                format!("unsupported trait derive: {ident}"),
            ));
        }
    };
    let spanned_trait = SpannedDeriveTrait {
        item: derive_trait,
        span: ident.span(),
    };
    Ok(spanned_trait)
}

#[derive(Debug)]
pub struct ParseableAttributes<Sanitizer, Validator> {
    pub sanitizers: Vec<Sanitizer>,
    pub validators: Vec<Validator>,
    pub new_unchecked: NewUnchecked,
    pub default: Option<Expr>,
}

// By some reason Default cannot be derive.
impl<Sanitizer, Validator> Default for ParseableAttributes<Sanitizer, Validator> {
    fn default() -> Self {
        Self {
            sanitizers: vec![],
            validators: vec![],
            new_unchecked: NewUnchecked::Off,
            default: None,
        }
    }
}

impl<Sanitizer: Parse, Validator: Parse> Parse for ParseableAttributes<Sanitizer, Validator> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = ParseableAttributes::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if ident == "sanitize" {
                if input.peek(Paren) {
                    let content;
                    parenthesized!(content in input);
                    let items = content.parse_terminated(Sanitizer::parse, Token![,])?;
                    attrs.sanitizers = items.into_iter().collect();
                } else {
                    let msg = concat!(
                        "`sanitize` must be used with parenthesis.\n",
                        "For example:\n\n",
                        "    sanitize(trim)\n\n"
                    );
                    return Err(syn::Error::new(ident.span(), msg));
                }
            } else if ident == "validate" {
                if input.peek(Paren) {
                    let content;
                    parenthesized!(content in input);
                    let items = content.parse_terminated(Validator::parse, Token![,])?;
                    attrs.validators = items.into_iter().collect();
                } else {
                    let msg = concat!(
                        "`validate` must be used with parenthesis.\n",
                        "For example:\n\n",
                        "    validate(max = 99)\n\n"
                    );
                    return Err(syn::Error::new(ident.span(), msg));
                }
            } else if ident == "default" {
                let _eq: Token![=] = input.parse()?;
                let default_expr: Expr = input.parse()?;
                attrs.default = Some(default_expr);
            } else if ident == "new_unchecked" {
                match_feature!("new_unchecked",
                    // The feature is not enabled, so we return an error
                    on => {
                        attrs.new_unchecked = NewUnchecked::On;
                    },
                    off => {
                        let msg = concat!(
                            "To generate ::new_unchecked() function, the feature `new_unchecked` of crate `nutype` needs to be enabled.\n",
                            "But you ought to know: generally, THIS IS A BAD IDEA.\nUse it only when you really need it."
                        );
                        return Err(syn::Error::new(ident.span(), msg));
                    }
                )
            } else {
                let msg = format!("Unknown attribute `{ident}`");
                return Err(syn::Error::new(ident.span(), msg));
            }

            // Parse `,` unless it's the end of the stream
            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        Ok(attrs)
    }
}

pub fn parse_number<T>(input: ParseStream) -> syn::Result<(T, Span)>
where
    T: FromStr,
{
    let mut number_str = String::with_capacity(16);
    if input.peek(Token![-]) {
        let _: Token![-] = input.parse()?;
        number_str.push('-');
    }

    let lit: Lit = input.parse()?;
    let lit_str = match &lit {
        Lit::Float(lf) => lf.to_string(),
        Lit::Int(li) => li.to_string(),
        _ => {
            let msg = "Expected number literal";
            return Err(syn::Error::new(lit.span(), msg));
        }
    };

    number_str.push_str(&lit_str.replace('_', ""));

    let number: T = number_str.parse::<T>().map_err(|_err| {
        let msg = format!("Expected {}, got `{}`", type_name::<T>(), number_str);
        syn::Error::new(lit.span(), msg)
    })?;

    Ok((number, lit.span()))
}
