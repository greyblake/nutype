use proc_macro2::TokenStream;

use crate::{
    common::models::{Guard, RawGuard},
    common::models::{Kind, SpannedItem},
};

// Sanitizer
//

#[derive(Debug)]
pub enum FloatSanitizer<T> {
    With(TokenStream),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedFloatSanitizer<T> = SpannedItem<FloatSanitizer<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FloatSanitizerKind {
    With,
}

impl std::fmt::Display for FloatSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for FloatSanitizer<T> {
    type Kind = FloatSanitizerKind;

    fn kind(&self) -> FloatSanitizerKind {
        match self {
            Self::With(_) => FloatSanitizerKind::With,
            Self::_Phantom(_) => {
                unreachable!("Kind::kind(): FloatSanitizer::_Phantom must not be used")
            }
        }
    }
}

// Validator
//

#[derive(Debug)]
pub enum FloatValidator<T> {
    Min(T),
    Max(T),
    With(TokenStream),
    Finite,
}

pub type SpannedFloatValidator<T> = SpannedItem<FloatValidator<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FloatValidatorKind {
    Min,
    Max,
    With,
    Finite,
}

impl std::fmt::Display for FloatValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::With => write!(f, "with"),
            Self::Finite => write!(f, "finite"),
        }
    }
}

impl<T> Kind for FloatValidator<T> {
    type Kind = FloatValidatorKind;

    fn kind(&self) -> FloatValidatorKind {
        match self {
            Self::Min(_) => FloatValidatorKind::Min,
            Self::Max(_) => FloatValidatorKind::Max,
            Self::With(_) => FloatValidatorKind::With,
            Self::Finite => FloatValidatorKind::Finite,
        }
    }
}

// Traits
//
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum FloatDeriveTrait {
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
    Borrow,
    Display,
    Default,
    Deref,

    // External crates
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    // Arbitrary,
}

pub type FloatRawGuard<T> = RawGuard<SpannedFloatSanitizer<T>, SpannedFloatValidator<T>>;
pub type FloatGuard<T> = Guard<FloatSanitizer<T>, FloatValidator<T>>;
