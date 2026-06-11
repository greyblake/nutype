use proc_macro2::Span;

use crate::common::{
    models::{CfgAttrEntry, DeriveTrait, SpannedDeriveTrait, TypeName, ValidatedDerives},
    validate::{validate_all_derive_traits, validate_numeric_guard},
};

use super::models::{IntegerDeriveTrait, IntegerGuard, IntegerRawGuard};

pub fn validate_integer_guard<T>(
    raw_guard: IntegerRawGuard<T>,
    type_name: &TypeName,
) -> Result<IntegerGuard<T>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_numeric_guard(raw_guard, type_name)
}

pub fn validate_integer_derive_traits(
    derive_traits: Vec<SpannedDeriveTrait>,
    has_validation: bool,
    cfg_attr_entries: &[CfgAttrEntry],
    maybe_default_value: &Option<syn::Expr>,
    type_name: &TypeName,
) -> Result<ValidatedDerives<IntegerDeriveTrait>, syn::Error> {
    validate_all_derive_traits(
        has_validation,
        derive_traits,
        cfg_attr_entries,
        maybe_default_value,
        type_name,
        to_integer_derive_trait,
    )
}

pub(crate) fn to_integer_derive_trait(
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
