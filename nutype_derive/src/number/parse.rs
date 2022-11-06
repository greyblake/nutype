use std::fmt::Debug;
use std::str::FromStr;

use crate::common::parse::{
    parse_nutype_attributes, parse_value_as, try_unwrap_group, try_unwrap_ident,
};
use proc_macro2::{TokenStream as TokenStream2, TokenTree};

use super::{
    models::{
        NewtypeNumberMeta, NumberSanitizer, NumberValidator, RawNewtypeNumberMeta,
        SpannedNumberSanitizer, SpannedNumberValidator,
    },
    validate::validate_number_meta,
};

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
) -> Result<Vec<SpannedNumberSanitizer<T>>, Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut output = vec![];
    let mut iter = stream.into_iter();

    match iter.next() {
        None => {}
        Some(token) => {
            let ident = try_unwrap_ident(token.clone())?;
            match ident.to_string().as_str() {
                "clamp" => {
                    let t = iter.next().expect("clamp() cannot be empty");
                    let span = token.span().join(t.span()).unwrap();

                    let group = try_unwrap_group(t)?;
                    let list: Vec<T> = parse_list_of_numbers(group.stream());
                    if list.len() == 2 {
                        let mut iter = list.into_iter();
                        let min = iter.next().unwrap();
                        let max = iter.next().unwrap();
                        let sanitizer = NumberSanitizer::Clamp { min, max };
                        output.push(SpannedNumberSanitizer {
                            span,
                            item: sanitizer,
                        });
                    } else {
                        let msg = "Invalid parameters for clamp()";
                        let error = syn::Error::new(span, msg);
                        return Err(vec![error]);
                    }
                }
                unknown_sanitizer => {
                    let msg = format!("Unknown number sanitizer: `{unknown_sanitizer}`");
                    let error = syn::Error::new(ident.span(), msg);
                    return Err(vec![error]);
                }
            }
        }
    }

    Ok(output)
}

fn parse_validate_attrs<T>(
    stream: TokenStream2,
) -> Result<Vec<SpannedNumberValidator<T>>, Vec<syn::Error>>
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
) -> Result<Option<(SpannedNumberValidator<T>, ITER)>, Vec<syn::Error>>
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
                    let parsed_validator = SpannedNumberValidator {
                        span: ident.span(),
                        item: validator,
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                "max" => {
                    let (value, iter) = parse_value_as(iter)?;
                    let validator = NumberValidator::Max(value);
                    let parsed_validator = SpannedNumberValidator {
                        span: ident.span(),
                        item: validator,
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

// TODO: Refactor
fn parse_list_of_numbers<T>(stream: TokenStream2) -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut output: Vec<T> = Vec::new();
    let mut cur = String::new();

    for token in stream.into_iter() {
        let t = token.to_string();
        if t == "," {
            if !cur.is_empty() {
                // TODO: result an Result and error
                let val: T = cur.parse().unwrap();
                output.push(val);
                cur = String::new();
            }
        } else {
            cur.push_str(&t);
        }
    }
    if !cur.is_empty() {
        // TODO: result an Result and error
        let val: T = cur.parse().unwrap();
        output.push(val);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::models::NumberSanitizer;

    #[test]
    fn test_parse_sanitize_attrs() {
        let tokens = quote::quote! {
            clamp(-4, 10)
        };
        let parsed_sanitizers = parse_sanitize_attrs::<i32>(tokens).unwrap();
        let sanitizers: Vec<_> = parsed_sanitizers.into_iter().map(|s| s.item).collect();
        assert_eq!(
            sanitizers,
            vec![NumberSanitizer::Clamp { min: -4, max: 10 }]
        );
    }
}
