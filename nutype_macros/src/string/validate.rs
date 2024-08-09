use kinded::Kinded;
use std::collections::HashSet;

use proc_macro2::Span;

use crate::{
    common::{
        models::{DeriveTrait, SpannedDeriveTrait, TypeName, ValueOrExpr},
        validate::{validate_duplicates, validate_guard, validate_traits_from_xor_try_from},
    },
    string::models::{StringGuard, StringRawGuard, StringSanitizer, StringValidator},
};

use super::models::{
    SpannedStringSanitizer, SpannedStringValidator, StringDeriveTrait, StringSanitizerKind,
};

pub fn validate_string_guard(
    raw_guard: StringRawGuard,
    type_name: &TypeName,
) -> Result<StringGuard, syn::Error> {
    validate_guard(
        raw_guard,
        type_name,
        validate_validators,
        validate_sanitizers,
    )
}

fn validate_validators(
    validators: Vec<SpannedStringValidator>,
) -> Result<Vec<StringValidator>, syn::Error> {
    // Check duplicates
    //
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validators `{kind}`.\nDon't worry, you still remain ingenious!")
    })?;

    // len_char_max VS len_char_min
    //
    let maybe_len_char_min = validators
        .iter()
        .flat_map(|v| match v.item {
            StringValidator::LenCharMin(ValueOrExpr::Value(len)) => Some((v.span, len)),
            _ => None,
        })
        .next();
    let maybe_len_char_max = validators
        .iter()
        .flat_map(|v| match v.item {
            StringValidator::LenCharMax(ValueOrExpr::Value(len)) => Some((v.span, len)),
            _ => None,
        })
        .next();
    if let (Some((_, len_char_min)), Some((len_char_max_span, len_char_max))) =
        (maybe_len_char_min, maybe_len_char_max)
    {
        if len_char_min > len_char_max {
            let msg = "`len_char_min` cannot be greater than `len_char_max`.\nDon't you find this obvious?";
            let err = syn::Error::new(len_char_max_span, msg);
            return Err(err);
        }
    }

    // Validate regex
    //
    #[cfg(feature = "regex")]
    for v in validators.iter() {
        if let StringValidator::Regex(ref regex_def) = v.item {
            regex_validation::validate_regex_def(regex_def, v.span)?;
        }
    }

    let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers(
    sanitizers: Vec<SpannedStringSanitizer>,
) -> Result<Vec<StringSanitizer>, syn::Error> {
    validate_duplicates(&sanitizers, |kind| {
        format!(
            "Duplicated sanitizer `{kind}`.\nYou're doing well, it's not that bad unless you forgot to call your mom!"
        )
    })?;

    // Validate lowercase VS uppercase
    let lowercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Lowercase);
    let uppercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Uppercase);
    if let (Some(lowercase), Some(uppercase)) = (lowercase, uppercase) {
        let msg = format!(
            "Using both sanitizers `{}` and `{}` makes no sense.\nYou're a great developer! Take care of yourself, a 5 mins break may help.",
            lowercase.kind(),
            uppercase.kind()
        );
        let span = lowercase.span;
        let err = syn::Error::new(span, msg);
        return Err(err);
    }

    let sanitizers: Vec<StringSanitizer> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_string_derive_traits(
    guard: &StringGuard,
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
) -> Result<HashSet<StringDeriveTrait>, syn::Error> {
    validate_traits_from_xor_try_from(&spanned_derive_traits)?;

    let mut traits = HashSet::with_capacity(24);
    let has_validation = guard.has_validation();

    for spanned_trait in spanned_derive_traits {
        let string_derive_trait =
            to_string_derive_trait(spanned_trait.item, has_validation, spanned_trait.span)?;
        traits.insert(string_derive_trait);
    }

    Ok(traits)
}

fn to_string_derive_trait(
    tr: DeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<StringDeriveTrait, syn::Error> {
    match tr {
        DeriveTrait::Debug => Ok(StringDeriveTrait::Debug),
        DeriveTrait::Display => Ok(StringDeriveTrait::Display),
        DeriveTrait::Default => Ok(StringDeriveTrait::Default),
        DeriveTrait::Clone => Ok(StringDeriveTrait::Clone),
        DeriveTrait::PartialEq => Ok(StringDeriveTrait::PartialEq),
        DeriveTrait::Eq => Ok(StringDeriveTrait::Eq),
        DeriveTrait::PartialOrd => Ok(StringDeriveTrait::PartialOrd),
        DeriveTrait::Ord => Ok(StringDeriveTrait::Ord),
        DeriveTrait::FromStr => Ok(StringDeriveTrait::FromStr),
        DeriveTrait::AsRef => Ok(StringDeriveTrait::AsRef),
        DeriveTrait::Deref => Ok(StringDeriveTrait::Deref),
        DeriveTrait::Hash => Ok(StringDeriveTrait::Hash),
        DeriveTrait::Borrow => Ok(StringDeriveTrait::Borrow),
        DeriveTrait::Into => Ok(StringDeriveTrait::Into),
        DeriveTrait::SerdeSerialize => Ok(StringDeriveTrait::SerdeSerialize),
        DeriveTrait::SerdeDeserialize => Ok(StringDeriveTrait::SerdeDeserialize),
        DeriveTrait::SchemarsJsonSchema => Ok(StringDeriveTrait::SchemarsJsonSchema),
        DeriveTrait::Copy => Err(syn::Error::new(
            span,
            "Copy trait cannot be derived for a String based type",
        )),
        DeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(
                    span,
                    "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead.",
                ))
            } else {
                Ok(StringDeriveTrait::From)
            }
        }
        DeriveTrait::TryFrom => Ok(StringDeriveTrait::TryFrom),
        DeriveTrait::ArbitraryArbitrary => Ok(StringDeriveTrait::ArbitraryArbitrary),
    }
}

#[cfg(feature = "regex")]
mod regex_validation {
    use super::*;
    use crate::string::models::RegexDef;

    pub fn validate_regex_def(regex_def: &RegexDef, span: Span) -> Result<(), syn::Error> {
        match regex_def {
            RegexDef::StringLiteral(lit) => {
                // Try to validate regex at compile time if it's a string literal
                let regex_str = lit.value();
                match regex::Regex::new(&regex_str) {
                    Ok(_re) => Ok(()),
                    Err(err) => Err(syn::Error::new(span, format!("{err}"))),
                }
            }
            RegexDef::Path(_) => Ok(()),
        }
    }
}
