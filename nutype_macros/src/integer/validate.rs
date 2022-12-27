use std::collections::HashSet;

use proc_macro2::Span;

use crate::{
    common::models::{DeriveTrait, NormalDeriveTrait, SpannedDeriveTrait},
    common::validate::validate_duplicates,
};

use super::models::{
    IntegerDeriveTrait, IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerValidator,
    SpannedIntegerSanitizer, SpannedIntegerValidator,
};

pub fn validate_number_meta<T>(raw_meta: IntegerRawGuard<T>) -> Result<IntegerGuard<T>, syn::Error>
where
    T: PartialOrd + Clone,
{
    let IntegerRawGuard {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(IntegerGuard::WithoutValidation { sanitizers })
    } else {
        Ok(IntegerGuard::WithValidation {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators<T>(
    validators: Vec<SpannedIntegerValidator<T>>,
) -> Result<Vec<IntegerValidator<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validator `{kind}`.\nYou're a great engineer, but don't forget to take care of yourself!")
    })?;

    // max VS min
    let maybe_min = validators
        .iter()
        .flat_map(|v| match &v.item {
            IntegerValidator::Min(ref min) => Some((v.span, min.clone())),
            _ => None,
        })
        .next();
    let maybe_max = validators
        .iter()
        .flat_map(|v| match v.item {
            IntegerValidator::Max(ref max) => Some((v.span, max.clone())),
            _ => None,
        })
        .next();
    if let (Some((_min_span, min)), Some((max_span, max))) = (maybe_min, maybe_max) {
        if min > max {
            let msg = "`min` cannot be greater than `max`.\nSometimes we all need a little break.";
            let err = syn::Error::new(max_span, msg);
            return Err(err);
        }
    }

    let validators: Vec<_> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers<T>(
    sanitizers: Vec<SpannedIntegerSanitizer<T>>,
) -> Result<Vec<IntegerSanitizer<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nIt happens, don't worry. We still love you!")
    })?;

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_integer_derive_traits(
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
    has_validation: bool,
) -> Result<HashSet<IntegerDeriveTrait>, syn::Error> {
    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits {
        match spanned_trait.item {
            DeriveTrait::Asterisk => {
                traits.extend(unfold_asterisk_traits(has_validation));
            }
            DeriveTrait::Normal(normal_trait) => {
                let string_derive_trait =
                    to_integer_derive_trait(normal_trait, has_validation, spanned_trait.span)?;
                traits.insert(string_derive_trait);
            }
        };
    }

    Ok(traits)
}

fn unfold_asterisk_traits(has_validation: bool) -> impl Iterator<Item = IntegerDeriveTrait> {
    let from_or_try_from = if has_validation {
        IntegerDeriveTrait::TryFrom
    } else {
        IntegerDeriveTrait::From
    };

    [
        from_or_try_from,
        IntegerDeriveTrait::Debug,
        IntegerDeriveTrait::Clone,
        IntegerDeriveTrait::Copy,
        IntegerDeriveTrait::PartialEq,
        IntegerDeriveTrait::Eq,
        IntegerDeriveTrait::PartialOrd,
        IntegerDeriveTrait::Ord,
        IntegerDeriveTrait::FromStr,
        IntegerDeriveTrait::AsRef,
        IntegerDeriveTrait::Hash,
    ]
    .into_iter()
}

fn to_integer_derive_trait(
    tr: NormalDeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<IntegerDeriveTrait, syn::Error> {
    match tr {
        NormalDeriveTrait::Debug => Ok(IntegerDeriveTrait::Debug),
        NormalDeriveTrait::Display => Ok(IntegerDeriveTrait::Display),
        NormalDeriveTrait::Clone => Ok(IntegerDeriveTrait::Clone),
        NormalDeriveTrait::PartialEq => Ok(IntegerDeriveTrait::PartialEq),
        NormalDeriveTrait::Eq => Ok(IntegerDeriveTrait::Eq),
        NormalDeriveTrait::PartialOrd => Ok(IntegerDeriveTrait::PartialOrd),
        NormalDeriveTrait::Ord => Ok(IntegerDeriveTrait::Ord),
        NormalDeriveTrait::Into => Ok(IntegerDeriveTrait::Into),
        NormalDeriveTrait::FromStr => Ok(IntegerDeriveTrait::FromStr),
        NormalDeriveTrait::AsRef => Ok(IntegerDeriveTrait::AsRef),
        NormalDeriveTrait::Hash => Ok(IntegerDeriveTrait::Hash),
        NormalDeriveTrait::Borrow => Ok(IntegerDeriveTrait::Borrow),
        NormalDeriveTrait::Copy => Ok(IntegerDeriveTrait::Copy),
        NormalDeriveTrait::SerdeSerialize => Ok(IntegerDeriveTrait::SerdeSerialize),
        NormalDeriveTrait::SerdeDeserialize => Ok(IntegerDeriveTrait::SerdeDeserialize),
        NormalDeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(span, "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead."))
            } else {
                Ok(IntegerDeriveTrait::From)
            }
        }
        NormalDeriveTrait::TryFrom => {
            if has_validation {
                Ok(IntegerDeriveTrait::TryFrom)
            } else {
                Err(syn::Error::new(span, "#[nutype] cannot derive `TryFrom`, because there is no validation. Use `From` instead."))
            }
        }
    }
}
