use proc_macro2::TokenStream;

use crate::{
    base::{Kind, SpannedItem},
    models::{NewtypeMeta, RawNewtypeMeta},
};

// Sanitizer
//

#[derive(Debug)]
pub enum IntegerSanitizer<T> {
    Clamp { min: T, max: T },
    With(TokenStream),
}

pub type SpannedIntegerSanitizer<T> = SpannedItem<IntegerSanitizer<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum IntegerSanitizerKind {
    Clamp,
    With,
}

impl std::fmt::Display for IntegerSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clamp => write!(f, "clamp"),
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for IntegerSanitizer<T> {
    type Kind = IntegerSanitizerKind;

    fn kind(&self) -> IntegerSanitizerKind {
        match self {
            Self::Clamp { .. } => IntegerSanitizerKind::Clamp,
            Self::With(_) => IntegerSanitizerKind::With,
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

#[derive(Debug, PartialEq, Eq)]
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
    // // External crates
    //
    // Serialize,
    // Deserialize,
    // Arbitrary,
}

// Meta
//

pub type RawNewtypeIntegerMeta<T> =
    RawNewtypeMeta<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>>;
pub type NewtypeIntegerMeta<T> = NewtypeMeta<IntegerSanitizer<T>, IntegerValidator<T>>;
