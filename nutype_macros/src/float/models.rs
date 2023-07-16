use proc_macro2::TokenStream;

use crate::common::models::{Guard, Kind, RawGuard, SpannedItem, TypedCustomFunction};

// Sanitizer
//

#[derive(Debug)]
pub enum FloatSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedFloatSanitizer<T> = SpannedItem<FloatSanitizer<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FloatSanitizerKind {
    With,
}

impl std::fmt::Display for FloatSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for FloatSanitizer<T> {
    type Kind = FloatSanitizerKind;

    fn kind(&self) -> FloatSanitizerKind {
        match self {
            Self::With(_) => FloatSanitizerKind::With,
            Self::_Phantom(_) => {
                unreachable!("Kind::kind(): FloatSanitizer::_Phantom must not be used")
            }
        }
    }
}

// Validator
//

#[derive(Debug)]
pub enum FloatValidator<T> {
    Min(T),
    Max(T),
    With(TypedCustomFunction),
    Finite,
}

pub type SpannedFloatValidator<T> = SpannedItem<FloatValidator<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FloatValidatorKind {
    Min,
    Max,
    With,
    Finite,
}

impl std::fmt::Display for FloatValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::With => write!(f, "with"),
            Self::Finite => write!(f, "finite"),
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
            Self::Finite => FloatValidatorKind::Finite,
        }
    }
}

// Traits
//
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum FloatDeriveTrait {
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
    Borrow,
    Display,
    Default,
    Deref,

    // External crates
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    // Arbitrary,
}

pub type FloatRawGuard<T> = RawGuard<SpannedFloatSanitizer<T>, SpannedFloatValidator<T>>;
pub type FloatGuard<T> = Guard<FloatSanitizer<T>, FloatValidator<T>>;

pub trait FloatType {
    fn float_inner_type() -> FloatInnerType;
}

macro_rules! define_float_inner_type {
    ($($tp:ty => $variant:ident),*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum FloatInnerType {
            $($variant),*
        }

        $(
            impl FloatType for $tp {
                fn float_inner_type() -> FloatInnerType {
                    FloatInnerType::$variant
                }
            }
        )*

        impl quote::ToTokens for FloatInnerType {
            fn to_tokens(&self, token_stream: &mut TokenStream) {
                let type_stream = match self {
                    $(
                        Self::$variant => quote::quote!($tp),
                    )*
                };
                type_stream.to_tokens(token_stream);
            }
        }
    }
}

define_float_inner_type!(
    f32 => F32,
    f64 => F64
);
