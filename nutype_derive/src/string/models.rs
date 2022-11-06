use proc_macro2::Span;

use crate::{
    base::{Kind, SpannedItem},
    models::{NewtypeMeta, RawNewtypeMeta},
};

// Sanitizer

pub type SpannedStringSanitizer = SpannedItem<StringSanitizer>;

#[derive(Debug, PartialEq)]
pub enum StringSanitizer {
    Trim,
    Lowercase,
    Uppercase,
}

impl Kind for StringSanitizer {
    type Kind = StringSanitizerKind;

    fn kind(&self) -> StringSanitizerKind {
        match self {
            Self::Trim => StringSanitizerKind::Trim,
            Self::Lowercase => StringSanitizerKind::Lowercase,
            Self::Uppercase => StringSanitizerKind::Uppercase,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StringSanitizerKind {
    Trim,
    Lowercase,
    Uppercase,
}

impl std::fmt::Display for StringSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trim => write!(f, "trim"),
            Self::Lowercase => write!(f, "lowercase"),
            Self::Uppercase => write!(f, "uppercase"),
        }
    }
}

// Validator
//

pub type SpannedStringValidator = SpannedItem<StringValidator>;

#[derive(Debug, PartialEq)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    Present,
}

impl Kind for StringValidator {
    type Kind = StringValidatorKind;

    fn kind(&self) -> StringValidatorKind {
        match self {
            Self::MinLen(_) => StringValidatorKind::MinLen,
            Self::MaxLen(_) => StringValidatorKind::MaxLen,
            Self::Present => StringValidatorKind::Present,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StringValidatorKind {
    MinLen,
    MaxLen,
    Present,
}

impl std::fmt::Display for StringValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinLen => write!(f, "min_len"),
            Self::MaxLen => write!(f, "max_len"),
            Self::Present => write!(f, "present"),
        }
    }
}

// Meta

pub type RawNewtypeStringMeta = RawNewtypeMeta<SpannedStringSanitizer, SpannedStringValidator>;
pub type NewtypeStringMeta = NewtypeMeta<StringSanitizer, StringValidator>;
