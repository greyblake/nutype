use std::{fmt::Debug, str::FromStr};

use proc_macro2::{Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::ToTokens;
use syn::DeriveInput;

use crate::models::{
    InnerType, NewtypeStringMeta, StringSanitizer, StringValidator, TypeNameAndInnerType,
};

// TODO: Parse visibility as well
pub fn parse_type_name_and_inner_type(token_stream: TokenStream2) -> TypeNameAndInnerType {
    let input: DeriveInput = syn::parse(token_stream.into()).unwrap();

    let type_name = input.ident;

    let data_struct = match &input.data {
        syn::Data::Struct(v) => v.clone(),
        _ => panic!("Expected syn::Data::Struct, got: {:?}", input.data),
    };

    let fields_unnamed = match data_struct.fields {
        syn::Fields::Unnamed(fu) => fu,
        _ => panic!(
            "Expected syn::Fields::Unnamed, got: {:?}",
            &data_struct.fields
        ),
    };

    let seg = fields_unnamed.unnamed.iter().next().unwrap();

    let type_path = match seg.ty.clone() {
        syn::Type::Path(tp) => tp,
        _ => panic!("Expected syn::Type::Path, got: {:?}", &seg.ty),
    };

    let type_path_str = type_path.into_token_stream().to_string();

    let inner_type = match type_path_str.as_ref() {
        "String" => InnerType::String,
        tp => panic!("Unsupported inner type: {}", tp),
    };

    TypeNameAndInnerType {
        type_name,
        inner_type,
    }
}

pub fn parse_attributes(input: TokenStream2) -> NewtypeStringMeta {
    let mut output = NewtypeStringMeta {
        sanitizers: vec![],
        validators: vec![],
    };

    let mut iter = input.into_iter();

    loop {
        let token = match iter.next() {
            Some(t) => t,
            None => {
                return output;
            }
        };

        let ident = unwrap_ident(token);

        match ident.to_string().as_ref() {
            "sanitize" => {
                let token = iter.next().unwrap();
                let group = unwrap_group(token);

                let sanitize_stream = group.stream();
                output.sanitizers = parse_sanitize_attrs(sanitize_stream);
            }
            "validate" => {
                let token = iter.next().unwrap();
                let group = unwrap_group(token);
                let validate_stream = group.stream();
                output.validators = parse_validate_attrs(validate_stream);
            }
            unknown => panic!("Unknown nutype option: {unknown}"),
        }
    }
}

fn parse_sanitize_attrs(stream: TokenStream2) -> Vec<StringSanitizer> {
    let mut output = vec![];
    for token in stream.into_iter() {
        match token {
            TokenTree::Ident(ident) => match ident.to_string().as_ref() {
                "trim" => output.push(StringSanitizer::Trim),
                "lowercase" => output.push(StringSanitizer::Lowecase),
                "uppercase" => output.push(StringSanitizer::Uppercase),
                unknown_sanitizer => panic!("Unkonwn sanitizer: {unknown_sanitizer}"),
            },
            _ => (),
        }
    }

    output
}

fn parse_validate_attrs(stream: TokenStream2) -> Vec<StringValidator> {
    let mut output = vec![];

    let mut token_iter = stream.into_iter();
    loop {
        match parse_validation_rule(token_iter) {
            Some((validator, rest_iter)) => {
                token_iter = rest_iter;
                output.push(validator);
            }
            None => {
                break;
            }
        }
    }

    output
}

fn parse_validation_rule<ITER: Iterator<Item = TokenTree>>(
    mut iter: ITER,
) -> Option<(StringValidator, ITER)> {
    match iter.next() {
        Some(mut token) => {
            // Skip punctuations between validators
            if token.to_string() == "," {
                token = iter.next().unwrap();
            }

            let ident = unwrap_ident(token);
            match ident.to_string().as_ref() {
                "max_len" => {
                    let (value, iter) = parse_value_as(iter);
                    Some((StringValidator::MaxLen(value), iter))
                }
                "min_len" => {
                    let (value, iter) = parse_value_as(iter);
                    Some((StringValidator::MinLen(value), iter))
                }
                "present" => Some((StringValidator::Present, iter)),
                validator => panic!("Unexpected validator rule `{validator}`"),
            }
        }
        None => None,
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

fn unwrap_ident(token: TokenTree) -> Ident {
    match token {
        TokenTree::Ident(ident) => ident,
        t => panic!("nutype: expected ident, got: {t}"),
    }
}

fn unwrap_group(token: TokenTree) -> Group {
    match token {
        TokenTree::Group(group) => group,
        t => panic!("nutype: expected group, got: {t}"),
    }
}
