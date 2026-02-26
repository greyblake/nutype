use proc_macro2::Span;
use std::collections::HashSet;

use crate::common::{
    models::{
        CfgAttrContent, CfgAttrEntry, DeriveTrait, SpannedDeriveTrait, TypeName, ValidatedDerives,
        Validation,
    },
    validate::{
        validate_all_derive_traits, validate_duplicates, validate_guard, validate_numeric_bounds,
    },
};

use super::models::{
    FloatDeriveTrait, FloatGuard, FloatRawGuard, FloatSanitizer, FloatValidator,
    FloatValidatorKind, SpannedFloatSanitizer, SpannedFloatValidator,
};

pub fn validate_float_guard<T>(
    raw_guard: FloatRawGuard<T>,
    type_name: &TypeName,
) -> Result<FloatGuard<T>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_guard(
        raw_guard,
        type_name,
        validate_validators,
        validate_sanitizers,
    )
}

fn validate_validators<T>(
    validators: Vec<SpannedFloatValidator<T>>,
) -> Result<Vec<FloatValidator<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&validators, |kind| {
        format!(
            "Duplicated validator `{kind}`.\nYou're a great engineer, but don't forget to take care of yourself!"
        )
    })?;

    validate_numeric_bounds(&validators)?;

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
        FloatGuard::WithValidation { validation, .. } => match validation {
            Validation::Custom { .. } => false,
            Validation::Standard { validators, .. } => validators
                .iter()
                .any(|v| v.kind() == FloatValidatorKind::Finite),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ValidationInfo {
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
    derive_traits: Vec<SpannedDeriveTrait>,
    guard: &FloatGuard<T>,
    cfg_attr_entries: &[CfgAttrEntry],
    maybe_default_value: &Option<syn::Expr>,
    type_name: &TypeName,
) -> Result<ValidatedDerives<FloatDeriveTrait>, syn::Error> {
    let validation = ValidationInfo::from_guard(guard);

    // Build union of all spanned traits for inter-trait dependency checks
    let mut all_spanned = derive_traits.clone();
    for entry in cfg_attr_entries {
        if let CfgAttrContent::Derive(ref traits) = entry.content {
            all_spanned.extend(traits.iter().cloned());
        }
    }

    // Convert all traits for dependency checks
    let mut all_typed = HashSet::with_capacity(24);
    for spanned_trait in all_spanned.iter() {
        let typed = to_float_derive_trait(spanned_trait.item, validation, spanned_trait.span)?;
        all_typed.insert(typed);
    }

    // Get a span of a given trait from the full union
    let get_span_for = |needle: DeriveTrait| -> Span {
        all_spanned
            .iter()
            .flat_map(|spanned_tr| {
                if spanned_tr.item == needle {
                    Some(spanned_tr.span)
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(Span::call_site)
    };

    // Validate inter trait dependencies on the union
    if all_typed.contains(&FloatDeriveTrait::Eq)
        && !all_typed.contains(&FloatDeriveTrait::PartialEq)
    {
        let span = get_span_for(DeriveTrait::Eq);
        let msg = "Trait Eq requires PartialEq.\nEvery expert was once a beginner.";
        return Err(syn::Error::new(span, msg));
    }
    if all_typed.contains(&FloatDeriveTrait::Ord) {
        if !all_typed.contains(&FloatDeriveTrait::PartialOrd) {
            let span = get_span_for(DeriveTrait::Ord);
            let msg = "Trait Ord requires PartialOrd.\nÜbung macht den Meister.";
            return Err(syn::Error::new(span, msg));
        } else if !all_typed.contains(&FloatDeriveTrait::Eq) {
            let span = get_span_for(DeriveTrait::Ord);
            let msg = "Trait Ord requires Eq.\nFestina lente.";
            return Err(syn::Error::new(span, msg));
        }
    }

    // Use shared helper for the rest (From XOR TryFrom, conversion)
    validate_all_derive_traits(
        validation.has_validation,
        derive_traits,
        cfg_attr_entries,
        maybe_default_value,
        type_name,
        |tr, _has_validation, span| to_float_derive_trait(tr, validation, span),
    )
}

pub(crate) fn to_float_derive_trait(
    tr: DeriveTrait,
    validation: ValidationInfo,
    span: Span,
) -> Result<FloatDeriveTrait, syn::Error> {
    match tr {
        DeriveTrait::Debug => Ok(FloatDeriveTrait::Debug),
        DeriveTrait::Display => Ok(FloatDeriveTrait::Display),
        DeriveTrait::Default => Ok(FloatDeriveTrait::Default),
        DeriveTrait::Clone => Ok(FloatDeriveTrait::Clone),
        DeriveTrait::PartialEq => Ok(FloatDeriveTrait::PartialEq),
        DeriveTrait::Into => Ok(FloatDeriveTrait::Into),
        DeriveTrait::Eq => {
            if validation.has_nan_validation {
                Ok(FloatDeriveTrait::Eq)
            } else {
                let msg = "To derive Eq trait on float-based type there must be validation that proves that inner value is not NaN.\nConsider adding:\n    validate(finite)";
                Err(syn::Error::new(span, msg))
            }
        }
        DeriveTrait::PartialOrd => Ok(FloatDeriveTrait::PartialOrd),
        DeriveTrait::Ord => {
            if validation.has_nan_validation {
                Ok(FloatDeriveTrait::Ord)
            } else {
                let msg = "To derive Ord trait on float-based type there must be validation that proves that inner value is not NaN.\nConsider adding:\n    validate(finite)";
                Err(syn::Error::new(span, msg))
            }
        }
        DeriveTrait::FromStr => Ok(FloatDeriveTrait::FromStr),
        DeriveTrait::AsRef => Ok(FloatDeriveTrait::AsRef),
        DeriveTrait::Deref => Ok(FloatDeriveTrait::Deref),
        DeriveTrait::Hash => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `Hash` trait for float types.",
        )),
        DeriveTrait::Borrow => Ok(FloatDeriveTrait::Borrow),
        DeriveTrait::Copy => Ok(FloatDeriveTrait::Copy),
        DeriveTrait::From => {
            if validation.has_validation {
                Err(syn::Error::new(
                    span,
                    "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead.",
                ))
            } else {
                Ok(FloatDeriveTrait::From)
            }
        }
        DeriveTrait::IntoIterator => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `IntoIterator` trait for float types. Inner type must be a collection type.",
        )),
        DeriveTrait::TryFrom => Ok(FloatDeriveTrait::TryFrom),
        DeriveTrait::SerdeSerialize => Ok(FloatDeriveTrait::SerdeSerialize),
        DeriveTrait::SerdeDeserialize => Ok(FloatDeriveTrait::SerdeDeserialize),
        DeriveTrait::SchemarsJsonSchema => Ok(FloatDeriveTrait::SchemarsJsonSchema),
        DeriveTrait::ArbitraryArbitrary => Ok(FloatDeriveTrait::ArbitraryArbitrary),
        DeriveTrait::ValuableValuable => Ok(FloatDeriveTrait::ValuableValuable),
    }
}
