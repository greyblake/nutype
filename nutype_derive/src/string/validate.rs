use std::collections::HashSet;

use crate::base::Kind;
use crate::common::validate::validate_duplicates;
use crate::models::{DeriveTrait, SpannedDeriveTrait, StringSanitizer, StringValidator};
use crate::string::models::NewtypeStringMeta;
use crate::string::models::RawNewtypeStringMeta;

use super::models::{
    SpannedStringSanitizer, SpannedStringValidator, StringDeriveTrait, StringSanitizerKind,
};

pub fn validate_string_meta(
    raw_meta: RawNewtypeStringMeta,
) -> Result<NewtypeStringMeta, syn::Error> {
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
) -> Result<Vec<StringValidator>, syn::Error> {
    // Check duplicates
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validators `{kind}`.\nDon't worry, you still remain ingenious!")
    })?;

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
    if let (Some((_min_len_span, min_len)), Some((max_len_span, max_len))) =
        (maybe_min_len, maybe_max_len)
    {
        if min_len > max_len {
            let msg = "min_len cannot be greater than max_len.\nDon't you find this obvious?";
            let span = max_len_span;
            let err = syn::Error::new(span, msg);
            return Err(err);
        }
    }

    let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers(
    sanitizers: Vec<SpannedStringSanitizer>,
) -> Result<Vec<StringSanitizer>, syn::Error> {
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nYou're doing well, it's not that bad unless you forgot to call your mom!")
    })?;

    // Validate lowercase VS uppercase
    let lowercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Lowercase);
    let uppercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Uppercase);
    if let (Some(lowercase), Some(uppercase)) = (lowercase, uppercase) {
        let msg = format!("Using both sanitizers `{}` and `{}` makes no sense.\nYou're a great developer! Take care of yourself, a 5 mins break may help.", lowercase.kind(), uppercase.kind());
        let span = lowercase.span;
        let err = syn::Error::new(span, msg);
        return Err(err);
    }

    let sanitizers: Vec<StringSanitizer> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_derive_traits(
    meta: &NewtypeStringMeta,
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
) -> Result<HashSet<StringDeriveTrait>, syn::Error> {
    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits {
        match spanned_trait.item {
            DeriveTrait::Asterisk => {
                traits.extend(
                    [
                        StringDeriveTrait::Debug,
                        StringDeriveTrait::Clone,
                        StringDeriveTrait::PartialEq,
                        StringDeriveTrait::Eq,
                        StringDeriveTrait::PartialOrd,
                        StringDeriveTrait::Ord,
                        StringDeriveTrait::FromStr,
                        StringDeriveTrait::AsRef,
                        StringDeriveTrait::Hash,
                        // TODO: should depend on features
                        // StringDeriveTrait::Serialize,
                        // StringDeriveTrait::Deserialize,
                        // StringDeriveTrait::Arbitrary,
                    ]
                    .iter(),
                );
                match meta {
                    NewtypeStringMeta::From { .. } => traits.insert(StringDeriveTrait::From),
                    NewtypeStringMeta::TryFrom { .. } => traits.insert(StringDeriveTrait::TryFrom),
                };
                true
            }
            DeriveTrait::Debug => traits.insert(StringDeriveTrait::Debug),
            DeriveTrait::Clone => traits.insert(StringDeriveTrait::Clone),
            DeriveTrait::PartialEq => traits.insert(StringDeriveTrait::PartialEq),
            DeriveTrait::Eq => traits.insert(StringDeriveTrait::Eq),
            DeriveTrait::PartialOrd => traits.insert(StringDeriveTrait::PartialOrd),
            DeriveTrait::Ord => traits.insert(StringDeriveTrait::Ord),
            DeriveTrait::FromStr => traits.insert(StringDeriveTrait::FromStr),
            DeriveTrait::AsRef => traits.insert(StringDeriveTrait::AsRef),
            DeriveTrait::Hash => traits.insert(StringDeriveTrait::Hash),
            DeriveTrait::Serialize => {
                unimplemented!("Serialize is not yet implemented");
                // traits.insert(StringDeriveTrait::Serialize)
            }
            DeriveTrait::Deserialize => {
                unimplemented!("Deserialize is not yet implemented");
                // traits.insert(StringDeriveTrait::Deserialize)
            }
            DeriveTrait::Arbitrary => {
                unimplemented!("Arbitrary is not yet implemented");
                // traits.insert(StringDeriveTrait::Arbitrary)
            }
            DeriveTrait::Copy => {
                let err = syn::Error::new(
                    spanned_trait.span,
                    "Copy trait cannot be derived for a String based type",
                );
                return Err(err);
            }
            DeriveTrait::From => {
                match meta {
                    NewtypeStringMeta::From { .. } => traits.insert(StringDeriveTrait::From),
                    NewtypeStringMeta::TryFrom { .. } => {
                        let err = syn::Error::new(
                            spanned_trait.span,
                            "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead.",
                        );
                        return Err(err);
                    }
                }
            }
            DeriveTrait::TryFrom => {
                match meta {
                    NewtypeStringMeta::From { .. } => {
                        // TODO: refactor: use closure
                        let err = syn::Error::new(
                            spanned_trait.span,
                            "#[nutype] cannot derive `TryFrom`, because there is no validation. Use `From` instead.",
                        );
                        return Err(err);
                    },
                    NewtypeStringMeta::TryFrom { .. } => traits.insert(StringDeriveTrait::TryFrom),
                }
            }
        };
    }

    Ok(traits)
}
