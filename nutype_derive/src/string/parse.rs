use crate::common::parse::{parse_value_as, try_unwrap_group, try_unwrap_ident};
use crate::models::{StringSanitizer, StringValidator};
use crate::string::models::NewtypeStringMeta;
use crate::string::models::RawNewtypeStringMeta;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};

use super::models::{
    ParsedStringSanitizer, ParsedStringValidator, StringSanitizerKind, StringValidatorKind,
};

pub fn parse_attributes(input: TokenStream2) -> Result<NewtypeStringMeta, Vec<syn::Error>> {
    let RawNewtypeStringMeta {
        sanitizers,
        validators,
    } = parse_raw_attributes(input)?;

    validate_validators(&validators)?;
    validate_sanitizers(&sanitizers)?;

    let sanitizers: Vec<StringSanitizer> = sanitizers.into_iter().map(|s| s.sanitizer).collect();
    let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.validator).collect();

    if validators.is_empty() {
        Ok(NewtypeStringMeta::From { sanitizers })
    } else {
        Ok(NewtypeStringMeta::TryFrom {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators(validators: &[ParsedStringValidator]) -> Result<(), Vec<syn::Error>> {
    // Check duplicates
    for (i1, v1) in validators.iter().enumerate() {
        for (i2, v2) in validators.iter().enumerate() {
            if i1 != i2 && v1.kind() == v2.kind() {
                let msg = format!(
                    "Duplicated validators `{}`.\nDon't worry, you still remain ingenious!",
                    v1.kind()
                );
                let span = v1.span.join(v2.span).unwrap();
                let err = syn::Error::new(span, &msg);
                return Err(vec![err]);
            }
        }
    }

    // max_len VS min_len
    let maybe_min_len = validators
        .iter()
        .flat_map(|v| match v.validator {
            StringValidator::MinLen(len) => Some((v.span, len)),
            _ => None,
        })
        .next();
    let maybe_max_len = validators
        .iter()
        .flat_map(|v| match v.validator {
            StringValidator::MaxLen(len) => Some((v.span, len)),
            _ => None,
        })
        .next();
    if let (Some((min_len_span, min_len)), Some((max_len_span, max_len))) =
        (maybe_min_len, maybe_max_len)
    {
        if min_len > max_len {
            let msg =
                format!("min_len cannot be greater than max_len.\nDon't you find this obvious?");
            let span = min_len_span.join(max_len_span).unwrap();
            let err = syn::Error::new(span, &msg);
            return Err(vec![err]);
        }
    }

    Ok(())
}

fn validate_sanitizers(sanitizers: &[ParsedStringSanitizer]) -> Result<(), Vec<syn::Error>> {
    // Check duplicates
    for (i1, san1) in sanitizers.iter().enumerate() {
        for (i2, san2) in sanitizers.iter().enumerate() {
            if i1 != i2 && san1.kind() == san2.kind() {
                let msg = format!("Duplicated sanitizer `{}`.\nYou're doing well, just don't forget to call your mom!", san1.kind());
                let span = san1.span.join(san2.span).unwrap();
                let err = syn::Error::new(span, &msg);
                return Err(vec![err]);
            }
        }
    }

    // Validate lowercase VS uppercase
    let lowercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Lowercase);
    let uppercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Uppercase);
    if let (Some(lowercase), Some(uppercase)) = (lowercase, uppercase) {
        let msg = format!("Using both sanitizers `{}` and `{}` makes no sense.\nYou're great developer! Take care of yourself, a 5 mins break may help.", lowercase.kind(), uppercase.kind());
        let span = lowercase.span.join(uppercase.span).unwrap();
        let err = syn::Error::new(span, &msg);
        return Err(vec![err]);
    }

    Ok(())
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
