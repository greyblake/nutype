use std::collections::HashSet;

use proc_macro2::Span;

use crate::common::{
    models::{DeriveTrait, SpannedDeriveTrait},
    validate::{validate_duplicates, validate_numeric_bounds},
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
        DeriveTrait::DieselNewType => Ok(IntegerDeriveTrait::DieselNewType),
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
    }
}
