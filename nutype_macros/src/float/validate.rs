use proc_macro2::Span;
use std::collections::HashSet;

use crate::common::{
    models::{DeriveTrait, SpannedDeriveTrait},
    validate::{validate_duplicates, validate_numeric_bounds, validate_traits_from_xor_try_from},
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
        FloatGuard::WithValidation { ref validators, .. } => validators
            .iter()
            .any(|v| v.kind() == FloatValidatorKind::Finite),
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
    validate_traits_from_xor_try_from(&spanned_derive_traits)?;

    let validation = ValidationInfo::from_guard(guard);
    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits.iter() {
        let normal_trait = spanned_trait.item;
        let string_derive_trait =
            to_float_derive_trait(normal_trait, validation, spanned_trait.span)?;
        traits.insert(string_derive_trait);
    }

    // Get a span of a given trait, so we can render a better message below
    // when we validate inter trait dependencies.
    let get_span_for = |needle: DeriveTrait| -> Span {
        spanned_derive_traits
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

    // Validate inter trait dependencies
    //
    if traits.contains(&FloatDeriveTrait::Eq) && !traits.contains(&FloatDeriveTrait::PartialEq) {
        let span = get_span_for(DeriveTrait::Eq);
        let msg = "Trait Eq requires PartialEq.\nEvery expert was once a beginner.";
        return Err(syn::Error::new(span, msg));
    }
    if traits.contains(&FloatDeriveTrait::Ord) {
        if !traits.contains(&FloatDeriveTrait::PartialOrd) {
            let span = get_span_for(DeriveTrait::Ord);
            let msg = "Trait Ord requires PartialOrd.\nÜbung macht den Meister.";
            return Err(syn::Error::new(span, msg));
        } else if !traits.contains(&FloatDeriveTrait::Eq) {
            let span = get_span_for(DeriveTrait::Ord);
            let msg = "Trait Ord requires Eq.\nFestina lente.";
            return Err(syn::Error::new(span, msg));
        }
    }

    Ok(traits)
}

fn to_float_derive_trait(
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
        DeriveTrait::TryFrom => Ok(FloatDeriveTrait::TryFrom),
        DeriveTrait::SerdeSerialize => Ok(FloatDeriveTrait::SerdeSerialize),
        DeriveTrait::SerdeDeserialize => Ok(FloatDeriveTrait::SerdeDeserialize),
        DeriveTrait::SchemarsJsonSchema => Ok(FloatDeriveTrait::SchemarsJsonSchema),
        DeriveTrait::ArbitraryArbitrary => Ok(FloatDeriveTrait::ArbitraryArbitrary),
    }
}
