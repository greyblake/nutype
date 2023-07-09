use std::collections::HashSet;

use proc_macro2::Span;

use crate::common::models::Kind;
use crate::common::models::{DeriveTrait, NormalDeriveTrait, SpannedDeriveTrait};
use crate::common::validate::validate_duplicates;
use crate::string::models::{StringGuard, StringRawGuard, StringSanitizer, StringValidator};

use super::models::{
    SpannedStringSanitizer, SpannedStringValidator, StringDeriveTrait, StringSanitizerKind,
};

pub fn validate_string_meta(raw_meta: StringRawGuard) -> Result<StringGuard, darling::Error> {
    let StringRawGuard {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(StringGuard::WithoutValidation { sanitizers })
    } else {
        Ok(StringGuard::WithValidation {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators(
    validators: Vec<SpannedStringValidator>,
) -> Result<Vec<StringValidator>, darling::Error> {
    // Check duplicates
    //
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validators `{kind}`.\nDon't worry, you still remain ingenious!")
    })?;

    // max_len VS min_len
    //
    let maybe_min_len = validators
        .iter()
        .flat_map(|v| match v.as_ref() {
            StringValidator::MinLen(len) => Some((v.span(), len)),
            _ => None,
        })
        .next();
    let maybe_max_len = validators
        .iter()
        .flat_map(|v| match v.as_ref() {
            StringValidator::MaxLen(len) => Some((v.span(), len)),
            _ => None,
        })
        .next();
    if let (Some((_min_len_span, min_len)), Some((max_len_span, max_len))) =
        (maybe_min_len, maybe_max_len)
    {
        if min_len > max_len {
            let msg = "min_len cannot be greater than max_len.\nDon't you find this obvious?";
            let span = max_len_span;
            let err = syn::Error::new(span, msg).into();
            return Err(err);
        }
    }

    // Validate regex
    //
    #[cfg(feature = "regex")]
    for v in validators.iter() {
        if let StringValidator::Regex(ref regex_def) = v.as_ref() {
            regex_validation::validate_regex_def(regex_def, v.span())?;
        }
    }

    let validators: Vec<StringValidator> = validators
        .into_iter()
        .map(|v| v.as_ref().to_owned())
        .collect();
    Ok(validators)
}

fn validate_sanitizers(
    sanitizers: Vec<SpannedStringSanitizer>,
) -> Result<Vec<StringSanitizer>, darling::Error> {
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
        let span = lowercase.span();
        let err = syn::Error::new(span, msg).into();
        return Err(err);
    }

    let sanitizers: Vec<StringSanitizer> = sanitizers
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect();
    Ok(sanitizers)
}

pub fn validate_string_derive_traits(
    guard: &StringGuard,
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
) -> Result<HashSet<StringDeriveTrait>, darling::Error> {
    let mut traits = HashSet::with_capacity(24);
    let has_validation = guard.has_validation();

    for spanned_trait in spanned_derive_traits {
        match spanned_trait.as_ref() {
            DeriveTrait::Asterisk => {
                traits.extend(unfold_asterisk_traits(has_validation));
            }
            DeriveTrait::Normal(normal_trait) => {
                let string_derive_trait = to_string_derive_trait(
                    normal_trait.to_owned(),
                    has_validation,
                    spanned_trait.span(),
                )?;
                traits.insert(string_derive_trait);
            }
        };
    }

    Ok(traits)
}

fn unfold_asterisk_traits(has_validation: bool) -> impl Iterator<Item = StringDeriveTrait> {
    let from_or_try_from = if has_validation {
        StringDeriveTrait::TryFrom
    } else {
        StringDeriveTrait::From
    };

    [
        from_or_try_from,
        StringDeriveTrait::Debug,
        StringDeriveTrait::Clone,
        StringDeriveTrait::PartialEq,
        StringDeriveTrait::Eq,
        StringDeriveTrait::PartialOrd,
        StringDeriveTrait::Ord,
        StringDeriveTrait::FromStr,
        StringDeriveTrait::AsRef,
        StringDeriveTrait::Hash,
    ]
    .into_iter()
}

fn to_string_derive_trait(
    tr: NormalDeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<StringDeriveTrait, darling::Error> {
    match tr {
        NormalDeriveTrait::Debug => Ok(StringDeriveTrait::Debug),
        NormalDeriveTrait::Display => Ok(StringDeriveTrait::Display),
        NormalDeriveTrait::Default => Ok(StringDeriveTrait::Default),
        NormalDeriveTrait::Clone => Ok(StringDeriveTrait::Clone),
        NormalDeriveTrait::PartialEq => Ok(StringDeriveTrait::PartialEq),
        NormalDeriveTrait::Eq => Ok(StringDeriveTrait::Eq),
        NormalDeriveTrait::PartialOrd => Ok(StringDeriveTrait::PartialOrd),
        NormalDeriveTrait::Ord => Ok(StringDeriveTrait::Ord),
        NormalDeriveTrait::FromStr => Ok(StringDeriveTrait::FromStr),
        NormalDeriveTrait::AsRef => Ok(StringDeriveTrait::AsRef),
        NormalDeriveTrait::Deref => Ok(StringDeriveTrait::Deref),
        NormalDeriveTrait::Hash => Ok(StringDeriveTrait::Hash),
        NormalDeriveTrait::Borrow => Ok(StringDeriveTrait::Borrow),
        NormalDeriveTrait::Into => Ok(StringDeriveTrait::Into),
        NormalDeriveTrait::SerdeSerialize => Ok(StringDeriveTrait::SerdeSerialize),
        NormalDeriveTrait::SerdeDeserialize => Ok(StringDeriveTrait::SerdeDeserialize),
        NormalDeriveTrait::SchemarsJsonSchema => Ok(StringDeriveTrait::SchemarsJsonSchema),
        NormalDeriveTrait::Copy => Err(syn::Error::new(
            span,
            "Copy trait cannot be derived for a String based type",
        )
        .into()),
        NormalDeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(span, "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead.").into())
            } else {
                Ok(StringDeriveTrait::From)
            }
        }
        NormalDeriveTrait::TryFrom => Ok(StringDeriveTrait::TryFrom),
    }
}

#[cfg(feature = "regex")]
mod regex_validation {
    use super::*;
    use crate::string::models::RegexDef;

    pub fn validate_regex_def(regex_def: &RegexDef, span: Span) -> Result<(), darling::Error> {
        match regex_def {
            RegexDef::StringLiteral(lit) => {
                // Try to validate regex at compile time if it's a string literal
                let regex_str = lit.value();
                match regex::Regex::new(&regex_str) {
                    Ok(_re) => Ok(()),
                    Err(err) => Err(syn::Error::new(span, format!("{err}")).into()),
                }
            }
            RegexDef::Path(_) => Ok(()),
        }
    }
}
