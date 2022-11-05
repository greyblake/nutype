use std::fmt::Debug;
use std::str::FromStr;

use crate::common::parse::{parse_value_as, try_unwrap_ident, parse_nutype_attributes};
use proc_macro2::{TokenStream as TokenStream2, TokenTree};

use super::{models::{
    NumberValidator, ParsedNumberSanitizer, ParsedNumberValidator, RawNewtypeNumberMeta, NewtypeNumberMeta,
}, validate::validate_number_meta};

pub fn parse_attributes<T>(input: TokenStream2) -> Result<NewtypeNumberMeta<T>, Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    parse_raw_attributes(input).and_then(validate_number_meta)
}


fn parse_raw_attributes<T>(input: TokenStream2) -> Result<RawNewtypeNumberMeta<T>, Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs<T>(
    stream: TokenStream2,
) -> Result<Vec<ParsedNumberSanitizer<T>>, Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    // TODO: Implement!
    let mut output = vec![];
    Ok(output)
}


fn parse_validate_attrs<T>(
    stream: TokenStream2,
) -> Result<Vec<ParsedNumberValidator<T>>, Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
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

fn parse_validation_rule<T, ITER>(
    mut iter: ITER,
) -> Result<Option<(ParsedNumberValidator<T>, ITER)>, Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
    ITER: Iterator<Item = TokenTree>,
{
    match iter.next() {
        Some(mut token) => {
            // Skip punctuations between validators
            if token.to_string() == "," {
                token = iter.next().unwrap();
            }

            let ident = try_unwrap_ident(token)?;
            match ident.to_string().as_ref() {
                "min" => {
                    let (value, iter) = parse_value_as(iter)?;
                    let validator = NumberValidator::Min(value);
                    let parsed_validator = ParsedNumberValidator {
                        span: ident.span(),
                        validator,
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                "max" => {
                    let (value, iter) = parse_value_as(iter)?;
                    let validator = NumberValidator::Max(value);
                    let parsed_validator = ParsedNumberValidator {
                        span: ident.span(),
                        validator,
                    };
                    Ok(Some((parsed_validator, iter)))
                }
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
