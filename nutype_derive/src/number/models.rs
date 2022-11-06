use crate::{
    base::{Kind, SpannedItem},
    models::{NewtypeMeta, RawNewtypeMeta},
};

// Sanitizer
//

#[derive(Debug, PartialEq, Eq)]
pub enum NumberSanitizer<T> {
    Clamp { min: T, max: T },
}

pub type SpannedNumberSanitizer<T> = SpannedItem<NumberSanitizer<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum NumberSanitizerKind {
    Clamp,
}

impl std::fmt::Display for NumberSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clamp => write!(f, "clamp"),
        }
    }
}

impl<T> Kind for NumberSanitizer<T> {
    type Kind = NumberSanitizerKind;

    fn kind(&self) -> NumberSanitizerKind {
        match self {
            Self::Clamp { .. } => NumberSanitizerKind::Clamp,
        }
    }
}

// Validator
//

#[derive(Debug, PartialEq, Eq)]
pub enum NumberValidator<T> {
    Min(T),
    Max(T),
}

pub type SpannedNumberValidator<T> = SpannedItem<NumberValidator<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum NumberValidatorKind {
    Min,
    Max,
}

impl std::fmt::Display for NumberValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
        }
    }
}

impl<T> Kind for NumberValidator<T> {
    type Kind = NumberValidatorKind;

    fn kind(&self) -> NumberValidatorKind {
        match self {
            Self::Min(_) => NumberValidatorKind::Min,
            Self::Max(_) => NumberValidatorKind::Max,
        }
    }
}

// Meta
//

pub type RawNewtypeNumberMeta<T> =
    RawNewtypeMeta<SpannedNumberSanitizer<T>, SpannedNumberValidator<T>>;
pub type NewtypeNumberMeta<T> = NewtypeMeta<NumberSanitizer<T>, NumberValidator<T>>;
