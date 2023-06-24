use proc_macro2::TokenStream;

use crate::{
    common::models::{Guard, RawGuard},
    common::models::{Kind, SpannedItem},
};

// Sanitizer
//

#[derive(Debug)]
pub enum IntegerSanitizer<T> {
    With(TokenStream),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedIntegerSanitizer<T> = SpannedItem<IntegerSanitizer<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerSanitizerKind {
    With,
}

impl std::fmt::Display for IntegerSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for IntegerSanitizer<T> {
    type Kind = IntegerSanitizerKind;

    fn kind(&self) -> IntegerSanitizerKind {
        match self {
            Self::With(_) => IntegerSanitizerKind::With,
            Self::_Phantom(_) => {
                unreachable!("Kind::kind(): IntegerSanitizer::_Phantom must not be used")
            }
        }
    }
}

// Validator
//

#[derive(Debug)]
pub enum IntegerValidator<T> {
    Min(T),
    Max(T),
    With(TokenStream),
}

pub type SpannedIntegerValidator<T> = SpannedItem<IntegerValidator<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerValidatorKind {
    Min,
    Max,
    With,
}

impl std::fmt::Display for IntegerValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for IntegerValidator<T> {
    type Kind = IntegerValidatorKind;

    fn kind(&self) -> IntegerValidatorKind {
        match self {
            Self::Min(_) => IntegerValidatorKind::Min,
            Self::Max(_) => IntegerValidatorKind::Max,
            Self::With(_) => IntegerValidatorKind::With,
        }
    }
}

// Traits
//
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum IntegerDeriveTrait {
    // Standard
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    FromStr,
    AsRef,
    Into,
    From,
    TryFrom,
    Hash,
    Borrow,
    Display,
    Default,

    // // External crates
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    // Arbitrary,
}

pub type IntegerRawGuard<T> = RawGuard<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>>;
pub type IntegerGuard<T> = Guard<IntegerSanitizer<T>, IntegerValidator<T>>;
