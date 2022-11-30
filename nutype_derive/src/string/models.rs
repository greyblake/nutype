use proc_macro2::TokenStream;

use crate::{
    base::{Kind, SpannedItem},
    models::{NewtypeMeta, RawNewtypeMeta},
};

// Sanitizer

pub type SpannedStringSanitizer = SpannedItem<StringSanitizer>;

#[derive(Debug)]
pub enum StringSanitizer {
    Trim,
    Lowercase,
    Uppercase,
    With(TokenStream),
}

impl Kind for StringSanitizer {
    type Kind = StringSanitizerKind;

    fn kind(&self) -> StringSanitizerKind {
        match self {
            Self::Trim => StringSanitizerKind::Trim,
            Self::Lowercase => StringSanitizerKind::Lowercase,
            Self::Uppercase => StringSanitizerKind::Uppercase,
            Self::With(_) => StringSanitizerKind::With,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringSanitizerKind {
    Trim,
    Lowercase,
    Uppercase,
    With,
}

impl std::fmt::Display for StringSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trim => write!(f, "trim"),
            Self::Lowercase => write!(f, "lowercase"),
            Self::Uppercase => write!(f, "uppercase"),
            Self::With => write!(f, "with"),
        }
    }
}

// Validator
//

pub type SpannedStringValidator = SpannedItem<StringValidator>;

#[derive(Debug)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    Present,
    With(TokenStream),
}

impl Kind for StringValidator {
    type Kind = StringValidatorKind;

    fn kind(&self) -> StringValidatorKind {
        match self {
            Self::MinLen(_) => StringValidatorKind::MinLen,
            Self::MaxLen(_) => StringValidatorKind::MaxLen,
            Self::Present => StringValidatorKind::Present,
            Self::With(_) => StringValidatorKind::With,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringValidatorKind {
    MinLen,
    MaxLen,
    Present,
    With,
}

impl std::fmt::Display for StringValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinLen => write!(f, "min_len"),
            Self::MaxLen => write!(f, "max_len"),
            Self::Present => write!(f, "present"),
            Self::With => write!(f, "with"),
        }
    }
}

// Traits
//
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StringDeriveTrait {
    // Standard library
    Debug,
    Clone,
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
    // Serialize,
    // Deserialize,
    // Arbitrary,
}

// Meta

pub type RawNewtypeStringMeta = RawNewtypeMeta<SpannedStringSanitizer, SpannedStringValidator>;
pub type NewtypeStringMeta = NewtypeMeta<StringSanitizer, StringValidator>;
