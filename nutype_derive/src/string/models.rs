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
            StringSanitizer::Trim => StringSanitizerKind::Trim,
            StringSanitizer::Lowercase => StringSanitizerKind::Lowercase,
            StringSanitizer::Uppercase => StringSanitizerKind::Uppercase,
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

#[derive(Debug, PartialEq)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    Present,
}

pub type RawNewtypeStringMeta = RawNewtypeMeta<ParsedStringSanitizer, StringValidator>;
pub type NewtypeStringMeta = NewtypeMeta<StringSanitizer, StringValidator>;
