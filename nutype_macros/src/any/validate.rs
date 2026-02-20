use proc_macro2::Span;

use crate::common::{
    models::{CfgAttrEntry, DeriveTrait, SpannedDeriveTrait, TypeName, ValidatedDerives},
    validate::{validate_duplicates, validate_guard},
};

use super::models::{
    AnyDeriveTrait, AnyGuard, AnyRawGuard, AnySanitizer, AnyValidator, SpannedAnySanitizer,
    SpannedAnyValidator,
};

pub fn validate_any_guard(
    raw_guard: AnyRawGuard,
    type_name: &TypeName,
) -> Result<AnyGuard, syn::Error> {
    validate_guard(
        raw_guard,
        type_name,
        validate_validators,
        validate_sanitizers,
    )
}

// > Quis custodiet ipsos custodes? :D
// (Who will guard the guards themselves?)
fn validate_validators(
    validators: Vec<SpannedAnyValidator>,
) -> Result<Vec<AnyValidator>, syn::Error> {
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validators `{kind}`.\nOh, maybe it's a time to take a break?")
    })?;

    let validators: Vec<AnyValidator> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers(
    sanitizers: Vec<SpannedAnySanitizer>,
) -> Result<Vec<AnySanitizer>, syn::Error> {
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nYou never know, what kind of error will be next!")
    })?;

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_any_derive_traits(
    guard: &AnyGuard,
    derive_traits: Vec<SpannedDeriveTrait>,
    cfg_attr_entries: &[CfgAttrEntry],
    maybe_default_value: &Option<syn::Expr>,
    type_name: &TypeName,
) -> Result<ValidatedDerives<AnyDeriveTrait>, syn::Error> {
    crate::common::validate::validate_all_derive_traits(
        guard.has_validation(),
        derive_traits,
        cfg_attr_entries,
        maybe_default_value,
        type_name,
        to_any_derive_trait,
    )
}

pub(crate) fn to_any_derive_trait(
    tr: DeriveTrait,
    _has_validation: bool,
    span: Span,
) -> Result<AnyDeriveTrait, syn::Error> {
    match tr {
        DeriveTrait::Debug => Ok(AnyDeriveTrait::Debug),
        DeriveTrait::Clone => Ok(AnyDeriveTrait::Clone),
        DeriveTrait::Copy => Ok(AnyDeriveTrait::Copy),
        DeriveTrait::PartialEq => Ok(AnyDeriveTrait::PartialEq),
        DeriveTrait::Eq => Ok(AnyDeriveTrait::Eq),
        DeriveTrait::Ord => Ok(AnyDeriveTrait::Ord),
        DeriveTrait::PartialOrd => Ok(AnyDeriveTrait::PartialOrd),
        DeriveTrait::Display => Ok(AnyDeriveTrait::Display),
        DeriveTrait::AsRef => Ok(AnyDeriveTrait::AsRef),
        DeriveTrait::Into => Ok(AnyDeriveTrait::Into),
        DeriveTrait::From => Ok(AnyDeriveTrait::From),
        DeriveTrait::Deref => Ok(AnyDeriveTrait::Deref),
        DeriveTrait::Borrow => Ok(AnyDeriveTrait::Borrow),
        DeriveTrait::FromStr => Ok(AnyDeriveTrait::FromStr),
        DeriveTrait::TryFrom => Ok(AnyDeriveTrait::TryFrom),
        DeriveTrait::Default => Ok(AnyDeriveTrait::Default),
        DeriveTrait::IntoIterator => Ok(AnyDeriveTrait::IntoIterator),
        DeriveTrait::SerdeSerialize => Ok(AnyDeriveTrait::SerdeSerialize),
        DeriveTrait::SerdeDeserialize => Ok(AnyDeriveTrait::SerdeDeserialize),
        DeriveTrait::Hash => Ok(AnyDeriveTrait::Hash),
        DeriveTrait::ArbitraryArbitrary => Ok(AnyDeriveTrait::ArbitraryArbitrary),
        DeriveTrait::ValuableValuable => Ok(AnyDeriveTrait::ValuableValuable),
        DeriveTrait::SchemarsJsonSchema => {
            let msg =
                format!("Deriving of trait `{tr:?}` is not (yet) supported for an arbitrary type");
            Err(syn::Error::new(span, msg))
        }
    }
}
