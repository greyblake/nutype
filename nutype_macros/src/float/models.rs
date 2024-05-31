use kinded::Kinded;
use proc_macro2::TokenStream;

use crate::common::models::{
    impl_numeric_bound_on_vec_of, impl_numeric_bound_validator, Guard, RawGuard, SpannedItem,
    TypeTrait, TypedCustomFunction, ValueOrExpr,
};

// Sanitizer
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case", derive(Hash))]
pub enum FloatSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedFloatSanitizer<T> = SpannedItem<FloatSanitizer<T>>;

// Validator
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case", derive(Hash))]
pub enum FloatValidator<T> {
    Greater(ValueOrExpr<T>),
    GreaterOrEqual(ValueOrExpr<T>),
    Less(ValueOrExpr<T>),
    LessOrEqual(ValueOrExpr<T>),
    Predicate(TypedCustomFunction),
    Finite,
}

impl_numeric_bound_validator!(FloatValidator);
impl_numeric_bound_on_vec_of!(FloatValidator);

pub type SpannedFloatValidator<T> = SpannedItem<FloatValidator<T>>;

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
    ArbitraryArbitrary,
}

impl TypeTrait for FloatDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &FloatDeriveTrait::FromStr
    }
}

pub type FloatRawGuard<T> = RawGuard<SpannedFloatSanitizer<T>, SpannedFloatValidator<T>>;
pub type FloatGuard<T> = Guard<FloatSanitizer<T>, FloatValidator<T>>;

pub trait FloatType {}

macro_rules! define_float_inner_type {
    ($($tp:ty => $variant:ident),*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum FloatInnerType {
            $($variant),*
        }

        $(
            impl FloatType for $tp {
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

        impl ::core::fmt::Display for FloatInnerType {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
                match self {
                    $(
                        Self::$variant => stringify!($tp).fmt(f),
                    )*
                }
            }
        }
    }
}

define_float_inner_type!(
    f32 => F32,
    f64 => F64
);
