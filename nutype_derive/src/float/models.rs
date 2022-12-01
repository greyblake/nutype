use proc_macro2::TokenStream;

use crate::{
    base::{Kind, SpannedItem},
    models::{NewtypeMeta, RawNewtypeMeta},
};

// Sanitizer
//

#[derive(Debug)]
pub enum FloatSanitizer<T> {
    Clamp { min: T, max: T },
    With(TokenStream),
}

pub type SpannedFloatSanitizer<T> = SpannedItem<FloatSanitizer<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum FloatSanitizerKind {
    Clamp,
    With,
}

impl std::fmt::Display for FloatSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clamp => write!(f, "clamp"),
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for FloatSanitizer<T> {
    type Kind = FloatSanitizerKind;

    fn kind(&self) -> FloatSanitizerKind {
        match self {
            Self::Clamp { .. } => FloatSanitizerKind::Clamp,
            Self::With(_) => FloatSanitizerKind::With,
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
}

pub type SpannedFloatValidator<T> = SpannedItem<FloatValidator<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum FloatValidatorKind {
    Min,
    Max,
    With,
}

impl std::fmt::Display for FloatValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::With => write!(f, "with"),
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
        }
    }
}

// Meta
//

pub type RawNewtypeFloatMeta<T> =
    RawNewtypeMeta<SpannedFloatSanitizer<T>, SpannedFloatValidator<T>>;
pub type NewtypeFloatMeta<T> = NewtypeMeta<FloatSanitizer<T>, FloatValidator<T>>;
