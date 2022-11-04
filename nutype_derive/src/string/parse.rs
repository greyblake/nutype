use crate::common::parse::{parse_value_as, try_unwrap_group, try_unwrap_ident};
use crate::models::{StringSanitizer, StringValidator};
use crate::string::models::NewtypeStringMeta;
use crate::string::models::RawNewtypeStringMeta;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};

use super::models::{
    ParsedStringSanitizer, ParsedStringValidator
};
use super::validate::validate_string_meta;

pub fn parse_attributes(input: TokenStream2) -> Result<NewtypeStringMeta, Vec<syn::Error>> {
    parse_raw_attributes(input).and_then(validate_string_meta)
}

fn parse_raw_attributes(input: TokenStream2) -> Result<RawNewtypeStringMeta, Vec<syn::Error>> {
    let mut output = RawNewtypeStringMeta {
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

fn parse_sanitize_attrs(
    stream: TokenStream2,
) -> Result<Vec<ParsedStringSanitizer>, Vec<syn::Error>> {
    let mut output = vec![];
    for token in stream.into_iter() {
        match token {
            TokenTree::Ident(ident) => {
                let san = match ident.to_string().as_ref() {
                    "trim" => StringSanitizer::Trim,
                    "lowercase" => StringSanitizer::Lowercase,
                    "uppercase" => StringSanitizer::Uppercase,
                    unknown_sanitizer => {
                        let msg = format!("Unknown sanitizer `{unknown_sanitizer}`");
                        let error = syn::Error::new(ident.span(), msg);
                        return Err(vec![error]);
                    }
                };
                output.push(ParsedStringSanitizer {
                    span: ident.span(),
                    sanitizer: san,
                });
            }
            _ => (),
        }
    }

    Ok(output)
}

fn parse_validate_attrs(
    stream: TokenStream2,
) -> Result<Vec<ParsedStringValidator>, Vec<syn::Error>> {
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
) -> Result<Option<(ParsedStringValidator, ITER)>, Vec<syn::Error>> {
    match iter.next() {
        Some(mut token) => {
            // Skip punctuations between validators
            if token.to_string() == "," {
                token = iter.next().unwrap();
            }

            let ident = try_unwrap_ident(token)?;
            match ident.to_string().as_ref() {
                "max_len" => {
                    let (value, iter) = parse_value_as(iter)?;
                    let validator = StringValidator::MaxLen(value);
                    let parsed_validator = ParsedStringValidator {
                        span: ident.span(),
                        validator,
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                "min_len" => {
                    let (value, iter) = parse_value_as(iter)?;
                    let validator = StringValidator::MinLen(value);
                    let parsed_validator = ParsedStringValidator {
                        span: ident.span(),
                        validator,
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                "present" => {
                    let validator = StringValidator::Present;
                    let parsed_validator = ParsedStringValidator {
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_validate_attrs() {
        let tokens = quote!(max_len = 13, min_len = 7, present);
        let validators = parse_validate_attrs(tokens).unwrap();
        let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.validator).collect();
        assert_eq!(
            validators,
            vec![
                StringValidator::MaxLen(13),
                StringValidator::MinLen(7),
                StringValidator::Present,
            ]
        );
    }

    #[test]
    fn test_validate_attrs_with_errors() {
        let tokens = quote!(max_len = -1);
        assert!(parse_validate_attrs(tokens).is_err());

        let tokens = quote!(present = 3);
        assert!(parse_validate_attrs(tokens).is_err());
    }
}
