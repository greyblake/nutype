use darling::util::SpannedValue;
use proc_macro2::TokenStream;

use crate::{
    common::models::Kind,
    common::models::{Guard, RawGuard},
};

// Sanitizer

pub type SpannedStringSanitizer = SpannedValue<StringSanitizer>;

#[derive(Debug, Default)]
pub struct SpannedStringSanitizers(pub Vec<SpannedStringSanitizer>);

#[derive(Debug, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

pub type SpannedStringValidator = SpannedValue<StringValidator>;

#[derive(Debug, Default)]
pub struct SpannedStringValidators(pub Vec<SpannedStringValidator>);

#[derive(Debug, Clone)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    NotEmpty,
    With(TokenStream),
    #[cfg_attr(not(feature = "regex"), allow(dead_code))]
    Regex(RegexDef),
}

#[cfg_attr(not(feature = "regex"), allow(dead_code))]
#[derive(Debug, Clone)]
pub enum RegexDef {
    /// The case, when regex is defined with string literal inlined, e.g.:
    ///     regex = "^[0-9]{9}$"
    StringLiteral(syn::LitStr),

    /// The case, when regex is with an ident, that refers to regex constant:
    ///     regex = SSN_REGEX
    Path(syn::Path),
}

impl Kind for StringValidator {
    type Kind = StringValidatorKind;

    fn kind(&self) -> StringValidatorKind {
        match self {
            Self::MinLen(_) => StringValidatorKind::MinLen,
            Self::MaxLen(_) => StringValidatorKind::MaxLen,
            Self::NotEmpty => StringValidatorKind::NotEmpty,
            Self::With(_) => StringValidatorKind::With,
            Self::Regex(_) => StringValidatorKind::Regex,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringValidatorKind {
    MinLen,
    MaxLen,
    NotEmpty,
    With,
    Regex,
}

impl std::fmt::Display for StringValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinLen => write!(f, "min_len"),
            Self::MaxLen => write!(f, "max_len"),
            Self::NotEmpty => write!(f, "not_empty"),
            Self::With => write!(f, "with"),
            Self::Regex => write!(f, "regex"),
        }
    }
}

// Traits
//
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StringDeriveTrait {
    // Standard
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
    Into,
    Hash,
    Borrow,
    Display,
    Default,
    Deref,

    // // External crates
    //
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    // Arbitrary,
}

pub type StringRawGuard = RawGuard<SpannedStringSanitizer, SpannedStringValidator>;
pub type StringGuard = Guard<StringSanitizer, StringValidator>;
