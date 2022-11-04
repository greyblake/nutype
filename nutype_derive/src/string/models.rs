use proc_macro2::Span;

use crate::models::{NewtypeMeta, RawNewtypeMeta};

// Sanitizer

#[derive(Debug)]
pub struct ParsedStringSanitizer {
    pub span: Span,
    pub sanitizer: StringSanitizer,
}

impl ParsedStringSanitizer {
    pub fn kind(&self) -> StringSanitizerKind {
        self.sanitizer.kind()
    }
}

#[derive(Debug, PartialEq)]
pub enum StringSanitizer {
    Trim,
    Lowercase,
    Uppercase,
}

impl StringSanitizer {
    pub fn kind(&self) -> StringSanitizerKind {
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
#[derive(Debug)]
pub struct ParsedStringValidator {
    pub span: Span,
    pub validator: StringValidator,
}

impl ParsedStringValidator {
    pub fn kind(&self) -> StringValidatorKind {
        self.validator.kind()
    }
}

#[derive(Debug, PartialEq)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    Present,
}

impl StringValidator {
    pub fn kind(&self) -> StringValidatorKind {
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

pub type RawNewtypeStringMeta = RawNewtypeMeta<ParsedStringSanitizer, ParsedStringValidator>;
pub type NewtypeStringMeta = NewtypeMeta<StringSanitizer, StringValidator>;
