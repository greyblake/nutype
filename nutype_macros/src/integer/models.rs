use kinded::Kinded;
use proc_macro2::TokenStream;

use crate::common::models::{
    impl_numeric_bound_validator, Guard, RawGuard, SpannedItem, TypeTrait, TypedCustomFunction,
};

// Sanitizer
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum IntegerSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(std::marker::PhantomData<T>),
}

pub type SpannedIntegerSanitizer<T> = SpannedItem<IntegerSanitizer<T>>;

// Validator
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum IntegerValidator<T> {
    Greater(T),
    GreaterOrEqual(T),
    Less(T),
    LessOrEqual(T),
    Predicate(TypedCustomFunction),
}

impl_numeric_bound_validator!(IntegerValidator);

pub type SpannedIntegerValidator<T> = SpannedItem<IntegerValidator<T>>;

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

impl TypeTrait for IntegerDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &IntegerDeriveTrait::FromStr
    }
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
