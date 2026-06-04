use proc_macro2::Span;

use crate::common::{
    models::{CfgAttrEntry, DeriveTrait, SpannedDeriveTrait, TypeName, ValidatedDerives},
    validate::{
        validate_all_derive_traits, validate_duplicates, validate_guard, validate_numeric_bounds,
    },
};

use super::models::{
    DecimalDeriveTrait, DecimalGuard, DecimalRawGuard, DecimalSanitizer, DecimalValidator,
    SpannedDecimalSanitizer, SpannedDecimalValidator,
};

pub fn validate_decimal_guard<T>(
    raw_guard: DecimalRawGuard<T>,
    type_name: &TypeName,
) -> Result<DecimalGuard<T>, syn::Error>
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
    validators: Vec<SpannedDecimalValidator<T>>,
) -> Result<Vec<DecimalValidator<T>>, syn::Error>
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
    sanitizers: Vec<SpannedDecimalSanitizer<T>>,
) -> Result<Vec<DecimalSanitizer<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nIt happens, don't worry. We still love you!")
    })?;

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_decimal_derive_traits(
    derive_traits: Vec<SpannedDeriveTrait>,
    has_validation: bool,
    cfg_attr_entries: &[CfgAttrEntry],
    maybe_default_value: &Option<syn::Expr>,
    type_name: &TypeName,
) -> Result<ValidatedDerives<DecimalDeriveTrait>, syn::Error> {
    validate_all_derive_traits(
        has_validation,
        derive_traits,
        cfg_attr_entries,
        maybe_default_value,
        type_name,
        to_decimal_derive_trait,
    )
}

pub(crate) fn to_decimal_derive_trait(
    tr: DeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<DecimalDeriveTrait, syn::Error> {
    match tr {
        DeriveTrait::Debug => Ok(DecimalDeriveTrait::Debug),
        DeriveTrait::Display => Ok(DecimalDeriveTrait::Display),
        DeriveTrait::Default => Ok(DecimalDeriveTrait::Default),
        DeriveTrait::Clone => Ok(DecimalDeriveTrait::Clone),
        DeriveTrait::PartialEq => Ok(DecimalDeriveTrait::PartialEq),
        DeriveTrait::Eq => Ok(DecimalDeriveTrait::Eq),
        DeriveTrait::PartialOrd => Ok(DecimalDeriveTrait::PartialOrd),
        DeriveTrait::Ord => Ok(DecimalDeriveTrait::Ord),
        DeriveTrait::Into => Ok(DecimalDeriveTrait::Into),
        DeriveTrait::FromStr => Ok(DecimalDeriveTrait::FromStr),
        DeriveTrait::AsRef => Ok(DecimalDeriveTrait::AsRef),
        DeriveTrait::Deref => Ok(DecimalDeriveTrait::Deref),
        DeriveTrait::Hash => Ok(DecimalDeriveTrait::Hash),
        DeriveTrait::Borrow => Ok(DecimalDeriveTrait::Borrow),
        DeriveTrait::Copy => Ok(DecimalDeriveTrait::Copy),
        DeriveTrait::SerdeSerialize => Ok(DecimalDeriveTrait::SerdeSerialize),
        DeriveTrait::SerdeDeserialize => Ok(DecimalDeriveTrait::SerdeDeserialize),
        DeriveTrait::ArbitraryArbitrary => Ok(DecimalDeriveTrait::ArbitraryArbitrary),
        DeriveTrait::TryFrom => Ok(DecimalDeriveTrait::TryFrom),
        DeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(
                    span,
                    "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead.",
                ))
            } else {
                Ok(DecimalDeriveTrait::From)
            }
        }
        DeriveTrait::IntoIterator => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `IntoIterator` trait for decimal types. Inner type must be a collection type.",
        )),
        DeriveTrait::SchemarsJsonSchema => Err(syn::Error::new(
            span,
            "#[nutype] does not support deriving `JsonSchema` for `rust_decimal::Decimal` yet.",
        )),
        DeriveTrait::ValuableValuable => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `Valuable` trait for `rust_decimal::Decimal`, because it does not implement `valuable::Valuable`.",
        )),
    }
}
