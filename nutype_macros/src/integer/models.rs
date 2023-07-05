use darling::util::SpannedValue;
use proc_macro2::TokenStream;

use crate::{
    common::models::Kind,
    common::models::{Guard, RawGuard},
};

// Sanitizer
//

#[derive(Debug, Clone)]
pub enum IntegerSanitizer<T> {
    With(TokenStream),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedIntegerSanitizer<T> = SpannedValue<IntegerSanitizer<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerSanitizerKind {
    With,
}

impl std::fmt::Display for IntegerSanitizerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for IntegerSanitizer<T> {
    type Kind = IntegerSanitizerKind;

    fn kind(&self) -> IntegerSanitizerKind {
        match self {
            Self::With(_) => IntegerSanitizerKind::With,
            Self::_Phantom(_) => {
                unreachable!("Kind::kind(): IntegerSanitizer::_Phantom must not be used")
            }
        }
    }
}

// Validator
//

#[derive(Debug, Clone)]
pub enum IntegerValidator<T> {
    Min(T),
    Max(T),
    With(TokenStream),
}

pub type SpannedIntegerValidator<T> = SpannedValue<IntegerValidator<T>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerValidatorKind {
    Min,
    Max,
    With,
}

impl std::fmt::Display for IntegerValidatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::With => write!(f, "with"),
        }
    }
}

impl<T> Kind for IntegerValidator<T> {
    type Kind = IntegerValidatorKind;

    fn kind(&self) -> IntegerValidatorKind {
        match self {
            Self::Min(_) => IntegerValidatorKind::Min,
            Self::Max(_) => IntegerValidatorKind::Max,
            Self::With(_) => IntegerValidatorKind::With,
        }
    }
}

// Traits
//
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum IntegerDeriveTrait {
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
    Hash,
    Borrow,
    Display,
    Default,
    Deref,

    // // External crates
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    // Arbitrary,
}

pub type IntegerRawGuard<T> = RawGuard<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>>;
pub type IntegerGuard<T> = Guard<IntegerSanitizer<T>, IntegerValidator<T>>;

pub trait IntegerType {
    fn integer_inner_type() -> IntegerInnerType;
}

macro_rules! define_integer_inner_type {
    ($($tp:ty => $variant:ident),*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum IntegerInnerType {
            $($variant),*
        }

        $(
            impl IntegerType for $tp {
                fn integer_inner_type() -> IntegerInnerType {
                    IntegerInnerType::$variant
                }
            }
        )*

        impl quote::ToTokens for IntegerInnerType {
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

define_integer_inner_type!(
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    usize => Usize,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
    isize => Isize
);
