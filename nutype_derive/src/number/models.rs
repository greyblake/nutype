use crate::models::{NewtypeMeta, RawNewtypeMeta};
use proc_macro2::Span;

// Sanitizer
//

#[derive(Debug, PartialEq)]
pub enum NumberSanitizer<T> {
    Clamp { min: T, max: T },
}

#[derive(Debug)]
pub struct ParsedNumberSanitizer<T> {
    pub span: Span,
    pub sanitizer: NumberSanitizer<T>,
}

#[derive(Debug, PartialEq)]
enum NumberSanitizerKind {
    Clamp,
}

impl std::fmt::Display for NumberSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clamp => write!(f, "clamp"),
        }
    }
}

impl<T> NumberSanitizer<T> {
    pub fn kind(&self) -> NumberSanitizerKind {
        match self {
            Self::Clamp { .. } => NumberSanitizerKind::Clamp,
        }
    }
}

impl<T> ParsedNumberSanitizer<T> {
    pub fn kind(&self) -> NumberSanitizerKind {
        self.sanitizer.kind()
    }
}

// Validator
//

#[derive(Debug, PartialEq)]
pub enum NumberValidator<T> {
    Min(T),
    Max(T),
}

#[derive(Debug)]
pub struct ParsedNumberValidator<T> {
    pub span: Span,
    pub validator: NumberValidator<T>,
}

#[derive(Debug, PartialEq)]
enum NumberValidatorKind {
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

impl<T> NumberValidator<T> {
    pub fn kind(&self) -> NumberValidatorKind {
        match self {
            Self::Min(_) => NumberValidatorKind::Min,
            Self::Max(_) => NumberValidatorKind::Max,
        }
    }
}

impl<T> ParsedNumberValidator<T> {
    pub fn kind(&self) -> NumberValidatorKind {
        self.validator.kind()
    }
}

// Meta
//

pub type RawNewtypeNumberMeta<T> =
    RawNewtypeMeta<ParsedNumberSanitizer<T>, ParsedNumberValidator<T>>;
pub type NewtypeNumberMeta<T> = NewtypeMeta<NumberSanitizer<T>, NumberValidator<T>>;
