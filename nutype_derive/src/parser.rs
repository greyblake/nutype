use std::{fmt::Debug, str::FromStr};

use proc_macro2::{Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::ToTokens;
use syn::{spanned::Spanned, DeriveInput};

use crate::models::{
    InnerType, NewtypeStringMeta, StringSanitizer, StringValidator, TypeNameAndInnerType,
};

// TODO: Parse visibility as well
pub fn parse_type_name_and_inner_type(
    token_stream: TokenStream2,
) -> Result<TypeNameAndInnerType, Vec<syn::Error>> {
    let input: DeriveInput = syn::parse(token_stream.into()).unwrap();

    let type_name = input.ident.clone();

    let data_struct = match &input.data {
        syn::Data::Struct(v) => v.clone(),
        _ => {
            let error = syn::Error::new(
                input.span(),
                "#[nutype] can be used only with tuple structs.",
            );
            return Err(vec![error]);
        }
    };

    let fields_unnamed = match data_struct.fields {
        syn::Fields::Unnamed(fu) => fu,
        _ => {
            let error = syn::Error::new(
                input.span(),
                "#[nutype] can be used only with tuple structs.",
            );
            return Err(vec![error]);
        }
    };

    let seg = fields_unnamed.unnamed.iter().next().unwrap();

    let type_path = match seg.ty.clone() {
        syn::Type::Path(tp) => tp,
        _ => {
            let error = syn::Error::new(
                seg.span(),
                "#[nutype] requires a simple inner type (e.g. String, i32, etc.)",
            );
            return Err(vec![error]);
        }
    };

    let type_path_str = type_path.into_token_stream().to_string();

    let inner_type = match type_path_str.as_ref() {
        "String" => InnerType::String,
        tp => {
            let error = syn::Error::new(
                seg.span(),
                format!("#[nutype] does not support `{tp}` as inner type"),
            );
            return Err(vec![error]);
        }
    };

    Ok(TypeNameAndInnerType {
        type_name,
        inner_type,
    })
}

pub fn parse_attributes(input: TokenStream2) -> Result<NewtypeStringMeta, Vec<syn::Error>> {
    let mut output = NewtypeStringMeta {
        sanitizers: vec![],
        validators: vec![],
    };

    let mut iter = input.into_iter();

    loop {
        let token = match iter.next() {
            Some(t) => t,
            None => {
                return Ok(output);
            }
        };

        let ident = try_unwrap_ident(token)?;

        match ident.to_string().as_ref() {
            "sanitize" => {
                let token = iter.next().unwrap();
                let group = try_unwrap_group(token)?;

                let sanitize_stream = group.stream();
                output.sanitizers = parse_sanitize_attrs(sanitize_stream)?;
            }
            "validate" => {
                let token = iter.next().unwrap();
                let group = try_unwrap_group(token)?;
                let validate_stream = group.stream();
                output.validators = parse_validate_attrs(validate_stream)?;
            }
            unknown => {
                let msg = format!("Unknown #[nutype] option: `{unknown}`");
                let error = syn::Error::new(ident.span(), msg);
                return Err(vec![error]);
            }
        }
    }
}

fn parse_sanitize_attrs(stream: TokenStream2) -> Result<Vec<StringSanitizer>, Vec<syn::Error>> {
    let mut output = vec![];
    for token in stream.into_iter() {
        match token {
            TokenTree::Ident(ident) => match ident.to_string().as_ref() {
                "trim" => output.push(StringSanitizer::Trim),
                "lowercase" => output.push(StringSanitizer::Lowecase),
                "uppercase" => output.push(StringSanitizer::Uppercase),
                unknown_sanitizer => {
                    let msg = format!("Unknown sanitizer `{unknown_sanitizer}`");
                    let error = syn::Error::new(ident.span(), msg);
                    return Err(vec![error]);
                }
            },
            _ => (),
        }
    }

    Ok(output)
}

fn parse_validate_attrs(stream: TokenStream2) -> Result<Vec<StringValidator>, Vec<syn::Error>> {
    let mut output = vec![];

    let mut token_iter = stream.into_iter();
    loop {
        match parse_validation_rule(token_iter)? {
            Some((validator, rest_iter)) => {
                token_iter = rest_iter;
                output.push(validator);
            }
            None => {
                break;
            }
        }
    }

    Ok(output)
}

fn parse_validation_rule<ITER: Iterator<Item = TokenTree>>(
    mut iter: ITER,
) -> Result<Option<(StringValidator, ITER)>, Vec<syn::Error>> {
    match iter.next() {
        Some(mut token) => {
            // Skip punctuations between validators
            if token.to_string() == "," {
                token = iter.next().unwrap();
            }

            let ident = try_unwrap_ident(token)?;
            match ident.to_string().as_ref() {
                "max_len" => {
                    let (value, iter) = parse_value_as(iter);
                    Ok(Some((StringValidator::MaxLen(value), iter)))
                }
                "min_len" => {
                    let (value, iter) = parse_value_as(iter);
                    Ok(Some((StringValidator::MinLen(value), iter)))
                }
                "present" => Ok(Some((StringValidator::Present, iter))),
                validator => {
                    let msg = format!("Unknown validation rule `{validator}`");
                    let error = syn::Error::new(ident.span(), msg);
                    return Err(vec![error]);
                }
            }
        }
        None => Ok(None),
    }
}

/// ## Example
/// Input (token stream):
///     = 123
/// Output (parsed value):
///    123
fn parse_value_as<T, ITER>(mut iter: ITER) -> (T, ITER)
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
    ITER: Iterator<Item = TokenTree>,
{
    let token_eq = iter.next().expect("Expected token `=`");
    assert_eq!(token_eq.to_string(), "=", "Expected token `=`");

    let token_value = iter.next().expect("Expected number");
    let str_value = token_value.to_string();
    let value: T = str_value
        .parse()
        .expect("Unexpected type of value for a validator");
    (value, iter)
}

fn try_unwrap_ident(token: TokenTree) -> Result<Ident, Vec<syn::Error>> {
    match token {
        TokenTree::Ident(ident) => Ok(ident),
        _ => {
            let error = syn::Error::new(token.span(), "#[nutype] expected ident");
            Err(vec![error])
        }
    }
}

fn try_unwrap_group(token: TokenTree) -> Result<Group, Vec<syn::Error>> {
    match token {
        TokenTree::Group(group) => Ok(group),
        _ => {
            let error = syn::Error::new(token.span(), "#[nutype] expected ident");
            Err(vec![error])
        }
    }
}
