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
#[kinded(display = "snake_case", derive(Hash))]
pub enum FloatSanitizer<T> {
    With(TypedCustomFunction),
    _Phantom(core::marker::PhantomData<T>),
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

impl<T: ToTokens> NumericValidatorTokens for FloatValidator<T> {
    fn view(&self) -> NumericValidatorView<'_> {
        match self {
            FloatValidator::Greater(v) => NumericValidatorView::Greater(v),
            FloatValidator::GreaterOrEqual(v) => NumericValidatorView::GreaterOrEqual(v),
            FloatValidator::Less(v) => NumericValidatorView::Less(v),
            FloatValidator::LessOrEqual(v) => NumericValidatorView::LessOrEqual(v),
            FloatValidator::Predicate(f) => NumericValidatorView::Predicate(f),
            FloatValidator::Finite => NumericValidatorView::Finite,
        }
    }
}

impl<T> NumericSanitizerTokens for FloatSanitizer<T> {
    fn custom_fn(&self) -> Option<&dyn ToTokens> {
        match self {
            FloatSanitizer::With(f) => Some(f),
            FloatSanitizer::_Phantom(_) => None,
        }
    }
}

pub type SpannedFloatValidator<T> = SpannedItem<FloatValidator<T>>;

// Traits
//
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
    ValuableValuable,
}

impl TypeTrait for FloatDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &FloatDeriveTrait::FromStr
    }
    fn is_default(&self) -> bool {
        self == &FloatDeriveTrait::Default
    }
}

pub type FloatRawGuard<T> = RawGuard<SpannedFloatSanitizer<T>, SpannedFloatValidator<T>>;
pub type FloatGuard<T> = Guard<FloatSanitizer<T>, FloatValidator<T>>;

pub trait FloatType {}

define_numeric_inner_type!(
    FloatInnerType, FloatType,
    f32 => F32,
    f64 => F64
);
