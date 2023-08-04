use kinded::Kinded;
use proc_macro2::TokenStream;

use crate::common::models::{Guard, RawGuard, SpannedItem, TypeTrait, TypedCustomFunction};

// Sanitizer
//

#[derive(Debug, Kinded)]
pub enum FloatSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedFloatSanitizer<T> = SpannedItem<FloatSanitizer<T>>;

impl std::fmt::Display for FloatSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::With => write!(f, "with"),
            Self::_Phantom => unreachable!("FloatSanitizerKind::_Phantom must not be used"),
        }
    }
}

// Validator
//

#[derive(Debug, Kinded)]
pub enum FloatValidator<T> {
    Min(T),
    Max(T),
    Predicate(TypedCustomFunction),
    Finite,
}

pub type SpannedFloatValidator<T> = SpannedItem<FloatValidator<T>>;

impl std::fmt::Display for FloatValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::Predicate => write!(f, "predicate"),
            Self::Finite => write!(f, "finite"),
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

impl TypeTrait for FloatDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &FloatDeriveTrait::FromStr
    }
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
