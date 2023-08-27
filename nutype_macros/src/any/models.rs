use kinded::Kinded;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::fmt::Debug;
use syn::Field;

use crate::common::models::{CustomFunction, Guard, RawGuard, SpannedItem, TypeTrait};

/// Sanitizer for "any" type.
#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum AnySanitizer {
    With(CustomFunction),
}

pub type SpannedAnySanitizer = SpannedItem<AnySanitizer>;

/// Validator for "any" type.
#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum AnyValidator {
    Predicate(CustomFunction),
}

pub type SpannedAnyValidator = SpannedItem<AnyValidator>;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum AnyDeriveTrait {
    // Standard
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    AsRef,
    Into,
    From,
    Deref,
    Borrow,
    FromStr,
    TryFrom,
    // Default,

    // // External crates
    // SerdeSerialize,
    // SerdeDeserialize,
    // SchemarsJsonSchema,
    // // Arbitrary,
}

impl TypeTrait for AnyDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &AnyDeriveTrait::FromStr
    }
}

pub type AnyRawGuard = RawGuard<SpannedAnySanitizer, SpannedAnyValidator>;
pub type AnyGuard = Guard<AnySanitizer, AnyValidator>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyInnerType(Field);

impl AnyInnerType {
    pub fn new(field: Field) -> Self {
        Self(field)
    }
}

impl ToTokens for AnyInnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        self.0.to_tokens(token_stream)
    }
}
