use kinded::Kinded;
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::{
    generate::numeric::{NumericSanitizerTokens, NumericValidatorTokens, NumericValidatorView},
    models::{
        Guard, RawGuard, SpannedItem, TypeTrait, TypedCustomFunction, ValueOrExpr,
        define_numeric_inner_type, impl_numeric_bound_on_vec_of, impl_numeric_bound_validator,
    },
};

// Sanitizer
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum IntegerSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(core::marker::PhantomData<T>),
}

pub type SpannedIntegerSanitizer<T> = SpannedItem<IntegerSanitizer<T>>;

// Validator
//

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum IntegerValidator<T> {
    Greater(ValueOrExpr<T>),
    GreaterOrEqual(ValueOrExpr<T>),
    Less(ValueOrExpr<T>),
    LessOrEqual(ValueOrExpr<T>),
    Predicate(TypedCustomFunction),
}

impl_numeric_bound_validator!(IntegerValidator);
impl_numeric_bound_on_vec_of!(IntegerValidator);

impl<T: ToTokens> NumericValidatorTokens for IntegerValidator<T> {
    fn view(&self) -> NumericValidatorView<'_> {
        match self {
            IntegerValidator::Greater(v) => NumericValidatorView::Greater(v),
            IntegerValidator::GreaterOrEqual(v) => NumericValidatorView::GreaterOrEqual(v),
            IntegerValidator::Less(v) => NumericValidatorView::Less(v),
            IntegerValidator::LessOrEqual(v) => NumericValidatorView::LessOrEqual(v),
            IntegerValidator::Predicate(f) => NumericValidatorView::Predicate(f),
        }
    }
}

impl<T> NumericSanitizerTokens for IntegerSanitizer<T> {
    fn custom_fn(&self) -> Option<&dyn ToTokens> {
        match self {
            IntegerSanitizer::With(f) => Some(f),
            IntegerSanitizer::_Phantom(_) => None,
        }
    }
}

pub type SpannedIntegerValidator<T> = SpannedItem<IntegerValidator<T>>;

// Traits
//
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

    // External crates
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    ArbitraryArbitrary,
    ValuableValuable,
}

impl TypeTrait for IntegerDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &IntegerDeriveTrait::FromStr
    }
    fn is_default(&self) -> bool {
        self == &IntegerDeriveTrait::Default
    }
}

pub type IntegerRawGuard<T> = RawGuard<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>>;
pub type IntegerGuard<T> = Guard<IntegerSanitizer<T>, IntegerValidator<T>>;

pub trait IntegerType {}

define_numeric_inner_type!(
    IntegerInnerType, IntegerType,
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
