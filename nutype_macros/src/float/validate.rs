use std::collections::HashSet;

use proc_macro2::Span;

use crate::common::models::Kind;
use crate::{
    common::models::{DeriveTrait, NormalDeriveTrait, SpannedDeriveTrait},
    common::validate::validate_duplicates,
};

use super::models::{
    FloatDeriveTrait, FloatGuard, FloatRawGuard, FloatSanitizer, FloatValidator,
    FloatValidatorKind, SpannedFloatSanitizer, SpannedFloatValidator,
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

fn has_validation_against_nan<T>(guard: &FloatGuard<T>) -> bool {
    match guard {
        FloatGuard::WithoutValidation { .. } => false,
        FloatGuard::WithValidation { ref validators, .. } => validators
            .iter()
            .any(|v| v.kind() == FloatValidatorKind::Finite)
    }
}

#[derive(Debug, Clone, Copy)]
struct ValidationInfo {
    has_validation: bool,
    has_nan_validation: bool,
}

impl ValidationInfo {
    fn from_guard<T>(guard: &FloatGuard<T>) -> ValidationInfo {
        let has_validation = guard.has_validation();
        let has_nan_validation = has_validation_against_nan(guard);
        ValidationInfo {
            has_validation,
            has_nan_validation,
        }
    }
}

pub fn validate_float_derive_traits<T>(
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
    guard: &FloatGuard<T>,
) -> Result<HashSet<FloatDeriveTrait>, syn::Error> {
    let validation = ValidationInfo::from_guard(guard);

    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits.iter() {
        match spanned_trait.item {
            DeriveTrait::Asterisk => {
                traits.extend(unfold_asterisk_traits(validation));
            }
            DeriveTrait::Normal(normal_trait) => {
                let string_derive_trait =
                    to_float_derive_trait(normal_trait, validation, spanned_trait.span)?;
                traits.insert(string_derive_trait);
            }
        };
    }

    if traits.contains(&FloatDeriveTrait::Eq) && !traits.contains(&FloatDeriveTrait::PartialEq) {
        let eq_trait_span = spanned_derive_traits
            .iter()
            .flat_map(|tr| match tr.item {
                DeriveTrait::Normal(NormalDeriveTrait::Eq) => Some(tr.span),
                DeriveTrait::Normal(_) => None,
                DeriveTrait::Asterisk => None,
            })
            .next()
            .unwrap_or_else(Span::call_site);
        let msg = "Trait Eq requires PartialEq.\nEvery expert was once a beginner.";
        return Err(syn::Error::new(eq_trait_span, msg));
    }

    Ok(traits)
}

fn unfold_asterisk_traits(validation: ValidationInfo) -> Vec<FloatDeriveTrait> {
    let from_or_try_from = if validation.has_validation {
        FloatDeriveTrait::TryFrom
    } else {
        FloatDeriveTrait::From
    };

    let mut traits = vec![
        from_or_try_from,
        FloatDeriveTrait::Debug,
        FloatDeriveTrait::Clone,
        FloatDeriveTrait::Copy,
        FloatDeriveTrait::PartialEq,
        FloatDeriveTrait::PartialOrd,
        FloatDeriveTrait::FromStr,
        FloatDeriveTrait::AsRef,
    ];

    if validation.has_nan_validation {
        // TODO: push Ord here as well
        traits.push(FloatDeriveTrait::Eq)
    }

    traits
}

fn to_float_derive_trait(
    tr: NormalDeriveTrait,
    validation: ValidationInfo,
    span: Span,
) -> Result<FloatDeriveTrait, syn::Error> {
    match tr {
        NormalDeriveTrait::Debug => Ok(FloatDeriveTrait::Debug),
        NormalDeriveTrait::Display => Ok(FloatDeriveTrait::Display),
        NormalDeriveTrait::Clone => Ok(FloatDeriveTrait::Clone),
        NormalDeriveTrait::PartialEq => Ok(FloatDeriveTrait::PartialEq),
        NormalDeriveTrait::Into => Ok(FloatDeriveTrait::Into),
        NormalDeriveTrait::Eq => {
            if validation.has_nan_validation {
                Ok(FloatDeriveTrait::Eq)
            } else {
                let msg = "To derive Eq trait on float-based type there must be validation that proves that inner value is not NaN.\nConsider adding:\n    validate(finite)";
                Err(syn::Error::new(span, msg))
            }
        }
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
            if validation.has_validation {
                Err(syn::Error::new(span, "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead."))
            } else {
                Ok(FloatDeriveTrait::From)
            }
        }
        NormalDeriveTrait::TryFrom => {
            if validation.has_validation {
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
