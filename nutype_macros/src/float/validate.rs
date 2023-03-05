use std::collections::HashSet;

use proc_macro2::Span;

use crate::{
    common::models::{DeriveTrait, NormalDeriveTrait, SpannedDeriveTrait},
    common::validate::validate_duplicates,
};

use super::models::{
    FloatDeriveTrait, FloatGuard, FloatRawGuard, FloatSanitizer, FloatValidator,
    SpannedFloatSanitizer, SpannedFloatValidator,
};

pub fn validate_number_meta<T>(raw_meta: FloatRawGuard<T>) -> Result<FloatGuard<T>, syn::Error>
where
    T: PartialOrd + Clone,
{
    let FloatRawGuard {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(FloatGuard::WithoutValidation { sanitizers })
    } else {
        Ok(FloatGuard::WithValidation {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators<T>(
    validators: Vec<SpannedFloatValidator<T>>,
) -> Result<Vec<FloatValidator<T>>, syn::Error>
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
            FloatValidator::Min(ref min) => Some((v.span, min.clone())),
            _ => None,
        })
        .next();
    let maybe_max = validators
        .iter()
        .flat_map(|v| match v.item {
            FloatValidator::Max(ref max) => Some((v.span, max.clone())),
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
    sanitizers: Vec<SpannedFloatSanitizer<T>>,
) -> Result<Vec<FloatSanitizer<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nIt happens, don't worry. We still love you!")
    })?;

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_float_derive_traits(
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
    has_validation: bool,
) -> Result<HashSet<FloatDeriveTrait>, syn::Error> {
    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits {
        match spanned_trait.item {
            DeriveTrait::Asterisk => {
                traits.extend(unfold_asterisk_traits(has_validation));
            }
            DeriveTrait::Normal(normal_trait) => {
                let string_derive_trait =
                    to_float_derive_trait(normal_trait, has_validation, spanned_trait.span)?;
                traits.insert(string_derive_trait);
            }
        };
    }

    Ok(traits)
}

fn unfold_asterisk_traits(has_validation: bool) -> impl Iterator<Item = FloatDeriveTrait> {
    let from_or_try_from = if has_validation {
        FloatDeriveTrait::TryFrom
    } else {
        FloatDeriveTrait::From
    };

    [
        from_or_try_from,
        FloatDeriveTrait::Debug,
        FloatDeriveTrait::Clone,
        FloatDeriveTrait::Copy,
        FloatDeriveTrait::PartialEq,
        FloatDeriveTrait::PartialOrd,
        FloatDeriveTrait::FromStr,
        FloatDeriveTrait::AsRef,
    ]
    .into_iter()
}

fn to_float_derive_trait(
    tr: NormalDeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<FloatDeriveTrait, syn::Error> {
    match tr {
        NormalDeriveTrait::Debug => Ok(FloatDeriveTrait::Debug),
        NormalDeriveTrait::Display => Ok(FloatDeriveTrait::Display),
        NormalDeriveTrait::Clone => Ok(FloatDeriveTrait::Clone),
        NormalDeriveTrait::PartialEq => Ok(FloatDeriveTrait::PartialEq),
        NormalDeriveTrait::Into => Ok(FloatDeriveTrait::Into),
        NormalDeriveTrait::Eq => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `Eq` trait for float types.",
        )),
        NormalDeriveTrait::PartialOrd => Ok(FloatDeriveTrait::PartialOrd),
        NormalDeriveTrait::Ord => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `Ord` trait for float types.",
        )),
        NormalDeriveTrait::FromStr => Ok(FloatDeriveTrait::FromStr),
        NormalDeriveTrait::AsRef => Ok(FloatDeriveTrait::AsRef),
        NormalDeriveTrait::Hash => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `Hash` trait for float types.",
        )),
        NormalDeriveTrait::Borrow => Ok(FloatDeriveTrait::Borrow),
        NormalDeriveTrait::Copy => Ok(FloatDeriveTrait::Copy),
        NormalDeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(span, "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead."))
            } else {
                Ok(FloatDeriveTrait::From)
            }
        }
        NormalDeriveTrait::TryFrom => {
            if has_validation {
                Ok(FloatDeriveTrait::TryFrom)
            } else {
                Err(syn::Error::new(span, "#[nutype] cannot derive `TryFrom`, because there is no validation. Use `From` instead."))
            }
        }
        NormalDeriveTrait::SerdeSerialize => Ok(FloatDeriveTrait::SerdeSerialize),
        NormalDeriveTrait::SerdeDeserialize => Ok(FloatDeriveTrait::SerdeDeserialize),
        NormalDeriveTrait::SchemarsJsonSchema => Ok(FloatDeriveTrait::SchemarsJsonSchema),
    }
}
