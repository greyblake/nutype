use crate::base::Kind;
use crate::models::{StringSanitizer, StringValidator};
use crate::string::models::NewtypeStringMeta;
use crate::string::models::RawNewtypeStringMeta;

use super::models::{SpannedStringSanitizer, SpannedStringValidator, StringSanitizerKind};

pub fn validate_string_meta(
    raw_meta: RawNewtypeStringMeta,
) -> Result<NewtypeStringMeta, Vec<syn::Error>> {
    let RawNewtypeStringMeta {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(NewtypeStringMeta::From { sanitizers })
    } else {
        Ok(NewtypeStringMeta::TryFrom {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators(
    validators: Vec<SpannedStringValidator>,
) -> Result<Vec<StringValidator>, Vec<syn::Error>> {
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
        .flat_map(|v| match v.item {
            StringValidator::MinLen(len) => Some((v.span, len)),
            _ => None,
        })
        .next();
    let maybe_max_len = validators
        .iter()
        .flat_map(|v| match v.item {
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

    let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers(
    sanitizers: Vec<SpannedStringSanitizer>,
) -> Result<Vec<StringSanitizer>, Vec<syn::Error>> {
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

    let sanitizers: Vec<StringSanitizer> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}
