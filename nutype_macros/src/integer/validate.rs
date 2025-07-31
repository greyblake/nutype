use std::collections::HashSet;

use proc_macro2::Span;

use crate::common::{
    models::{DeriveTrait, SpannedDeriveTrait, TypeName},
    validate::{
        validate_duplicates, validate_guard, validate_numeric_bounds,
        validate_traits_from_xor_try_from,
    },
};

use super::models::{
    IntegerDeriveTrait, IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerValidator,
    SpannedIntegerSanitizer, SpannedIntegerValidator,
};

pub fn validate_integer_guard<T>(
    raw_guard: IntegerRawGuard<T>,
    type_name: &TypeName,
) -> Result<IntegerGuard<T>, syn::Error>
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
    validators: Vec<SpannedIntegerValidator<T>>,
) -> Result<Vec<IntegerValidator<T>>, syn::Error>
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
    validate_traits_from_xor_try_from(&spanned_derive_traits)?;

    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits {
        let string_derive_trait =
            to_integer_derive_trait(spanned_trait.item, has_validation, spanned_trait.span)?;
        traits.insert(string_derive_trait);
    }

    Ok(traits)
}

fn to_integer_derive_trait(
    tr: DeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<IntegerDeriveTrait, syn::Error> {
    match tr {
        DeriveTrait::Debug => Ok(IntegerDeriveTrait::Debug),
        DeriveTrait::Display => Ok(IntegerDeriveTrait::Display),
        DeriveTrait::Default => Ok(IntegerDeriveTrait::Default),
        DeriveTrait::Clone => Ok(IntegerDeriveTrait::Clone),
        DeriveTrait::PartialEq => Ok(IntegerDeriveTrait::PartialEq),
        DeriveTrait::Eq => Ok(IntegerDeriveTrait::Eq),
        DeriveTrait::PartialOrd => Ok(IntegerDeriveTrait::PartialOrd),
        DeriveTrait::Ord => Ok(IntegerDeriveTrait::Ord),
        DeriveTrait::Into => Ok(IntegerDeriveTrait::Into),
        DeriveTrait::FromStr => Ok(IntegerDeriveTrait::FromStr),
        DeriveTrait::AsRef => Ok(IntegerDeriveTrait::AsRef),
        DeriveTrait::Deref => Ok(IntegerDeriveTrait::Deref),
        DeriveTrait::Hash => Ok(IntegerDeriveTrait::Hash),
        DeriveTrait::Borrow => Ok(IntegerDeriveTrait::Borrow),
        DeriveTrait::Copy => Ok(IntegerDeriveTrait::Copy),
        DeriveTrait::SerdeSerialize => Ok(IntegerDeriveTrait::SerdeSerialize),
        DeriveTrait::SerdeDeserialize => Ok(IntegerDeriveTrait::SerdeDeserialize),
        DeriveTrait::SchemarsJsonSchema => Ok(IntegerDeriveTrait::SchemarsJsonSchema),
        DeriveTrait::ArbitraryArbitrary => Ok(IntegerDeriveTrait::ArbitraryArbitrary),
        DeriveTrait::ValuableValuable => Ok(IntegerDeriveTrait::ValuableValuable),
        DeriveTrait::TryFrom => Ok(IntegerDeriveTrait::TryFrom),
        DeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(
                    span,
                    "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead.",
                ))
            } else {
                Ok(IntegerDeriveTrait::From)
            }
        }
        DeriveTrait::IntoIterator => Err(syn::Error::new(
            span,
            "#[nutype] cannot derive `IntoIterator` trait for integer types. Inner type must be a collection type.",
        )),
    }
}
