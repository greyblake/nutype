use proc_macro2::TokenStream;

use crate::{
    base::{Kind, SpannedItem},
    models::{NewtypeMeta, RawNewtypeMeta},
};

// Sanitizer
//

#[derive(Debug)]
pub enum NumberSanitizer<T> {
    Clamp { min: T, max: T },
    With(TokenStream),
}

pub type SpannedNumberSanitizer<T> = SpannedItem<NumberSanitizer<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum NumberSanitizerKind {
    Clamp,
    With,
}

impl std::fmt::Display for NumberSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clamp => write!(f, "clamp"),
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for NumberSanitizer<T> {
    type Kind = NumberSanitizerKind;

    fn kind(&self) -> NumberSanitizerKind {
        match self {
            Self::Clamp { .. } => NumberSanitizerKind::Clamp,
            Self::With(_) => NumberSanitizerKind::With,
        }
    }
}

// Validator
//

#[derive(Debug)]
pub enum NumberValidator<T> {
    Min(T),
    Max(T),
    With(TokenStream),
}

pub type SpannedNumberValidator<T> = SpannedItem<NumberValidator<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum NumberValidatorKind {
    Min,
    Max,
    With,
}

impl std::fmt::Display for NumberValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for NumberValidator<T> {
    type Kind = NumberValidatorKind;

    fn kind(&self) -> NumberValidatorKind {
        match self {
            Self::Min(_) => NumberValidatorKind::Min,
            Self::Max(_) => NumberValidatorKind::Max,
            Self::With(_) => NumberValidatorKind::With,
        }
    }
}

// Traits
//
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
    From,
    TryFrom,
    Hash,
    Borrow,
    // // External crates
    //
    // Serialize,
    // Deserialize,
    // Arbitrary,
}

// Meta
//

pub type RawNewtypeNumberMeta<T> =
    RawNewtypeMeta<SpannedNumberSanitizer<T>, SpannedNumberValidator<T>>;
pub type NewtypeNumberMeta<T> = NewtypeMeta<NumberSanitizer<T>, NumberValidator<T>>;
