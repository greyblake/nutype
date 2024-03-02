use kinded::Kinded;
use std::{collections::HashSet, fmt::Debug};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Attribute, ExprClosure, Path,
};

use crate::{
    any::models::AnyInnerType, float::models::FloatInnerType, integer::models::IntegerInnerType,
    string::models::StringInnerType,
};

use super::gen::type_custom_closure;

/// A spanned item. An item can be anything that cares a domain value.
/// Keeping a span allows to throw good precise error messages at the validation stage.
#[derive(Debug, Clone)]
pub struct SpannedItem<T> {
    pub item: T,
    pub span: Span,
}

impl<T> SpannedItem<T> {
    pub fn new(item: T, span: Span) -> Self {
        Self { item, span }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T: Kinded> Kinded for SpannedItem<T> {
    type Kind = <T as Kinded>::Kind;

    fn kind(&self) -> Self::Kind {
        self.item.kind()
    }
}

/// Represents the inner type of a newtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InnerType {
    String(StringInnerType),
    Integer(IntegerInnerType),
    Float(FloatInnerType),
    Any(AnyInnerType),
}

impl From<IntegerInnerType> for InnerType {
    fn from(tp: IntegerInnerType) -> InnerType {
        InnerType::Integer(tp)
    }
}

impl From<&IntegerInnerType> for InnerType {
    fn from(tp: &IntegerInnerType) -> InnerType {
        InnerType::Integer(*tp)
    }
}

impl From<FloatInnerType> for InnerType {
    fn from(tp: FloatInnerType) -> InnerType {
        InnerType::Float(tp)
    }
}

impl From<&FloatInnerType> for InnerType {
    fn from(tp: &FloatInnerType) -> InnerType {
        InnerType::Float(*tp)
    }
}

impl From<StringInnerType> for InnerType {
    fn from(string_inner_type: StringInnerType) -> InnerType {
        InnerType::String(string_inner_type)
    }
}

impl From<AnyInnerType> for InnerType {
    fn from(any_inner_type: AnyInnerType) -> InnerType {
        InnerType::Any(any_inner_type)
    }
}

impl From<&AnyInnerType> for InnerType {
    fn from(any_inner_type: &AnyInnerType) -> InnerType {
        InnerType::Any(any_inner_type.clone())
    }
}

impl ToTokens for InnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        match self {
            InnerType::String(string_type) => {
                string_type.to_tokens(token_stream);
            }
            InnerType::Integer(integer_type) => {
                integer_type.to_tokens(token_stream);
            }
            InnerType::Float(float_type) => {
                float_type.to_tokens(token_stream);
            }
            InnerType::Any(any_type) => {
                any_type.to_tokens(token_stream);
            }
        };
    }
}

macro_rules! define_ident_type {
    ($tp_name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $tp_name(proc_macro2::Ident);

        impl $tp_name {
            pub fn new(name: proc_macro2::Ident) -> Self {
                Self(name)
            }
        }

        impl core::fmt::Display for $tp_name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl ::quote::ToTokens for $tp_name {
            fn to_tokens(&self, token_stream: &mut TokenStream) {
                self.0.to_tokens(token_stream)
            }
        }
    };
}

// A type that represents a newtype name.
// For example: `Username`, `Email`, etc.
define_ident_type!(TypeName);

// Represents a type for a validation error.
// For example, if `TypeName` is `Email`, then `ErrorTypeName` would usually be `EmailError`.
define_ident_type!(ErrorTypeName);

// A type that represents an error name which is returned by `FromStr` traits.
// For example, if `TypeName` is `Amount`, then this would be `AmountParseError`.
define_ident_type!(ParseErrorTypeName);

// Module name, where the type is placed.
define_ident_type!(ModuleName);

#[derive(Debug)]
pub struct Meta {
    pub type_name: TypeName,
    pub inner_type: InnerType,
    pub vis: syn::Visibility,
    pub doc_attrs: Vec<Attribute>,
}

impl Meta {
    pub fn into_typed_meta(self, attrs: TokenStream) -> (TypedMeta, InnerType) {
        let Self {
            doc_attrs,
            type_name,
            inner_type,
            vis,
        } = self;
        let typed_meta = TypedMeta {
            doc_attrs,
            type_name,
            attrs,
            vis,
        };
        (typed_meta, inner_type)
    }
}

/// Meta information with attributes that can be given to Newtype::expand to generate an
/// implementation for a particular type.
#[derive(Debug)]
pub struct TypedMeta {
    pub type_name: TypeName,

    /// Attributes given to #[nutype] macro
    pub attrs: TokenStream,

    pub vis: syn::Visibility,
    pub doc_attrs: Vec<Attribute>,
}

/// Validated model, that represents precisely what needs to be generated.
#[derive(Debug)]
pub enum Guard<Sanitizer, Validator> {
    WithoutValidation {
        sanitizers: Vec<Sanitizer>,
    },
    WithValidation {
        sanitizers: Vec<Sanitizer>,
        validators: Vec<Validator>,
    },
}

/// Parsed attributes (`sanitize`, `validate`, `new_unchecked`).
#[derive(Debug)]
pub struct Attributes<G, DT> {
    /// Guard contains sanitizers and validators
    pub guard: G,

    /// `new_unchecked` flag
    pub new_unchecked: NewUnchecked,

    /// Value for Default trait. Provide with `default = `
    pub default: Option<syn::Expr>,

    pub derive_traits: Vec<DT>,
}

/// Represents a value known at compile time or an expression.
/// Knowing value at compile time allows to run some extra validations to prevent potential errors.
#[derive(Debug)]
pub enum ValueOrExpr<T> {
    Value(T),
    Expr(syn::Expr),
}

impl<T: ToTokens> ToTokens for ValueOrExpr<T> {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        match self {
            Self::Value(value) => {
                value.to_tokens(token_stream);
            }
            Self::Expr(expr) => {
                expr.to_tokens(token_stream);
            }
        };
    }
}

impl<Sanitizer, Validator> Guard<Sanitizer, Validator> {
    pub fn has_validation(&self) -> bool {
        match self {
            Self::WithoutValidation { .. } => false,
            Self::WithValidation { .. } => true,
        }
    }

    pub fn validators(&self) -> Option<&Vec<Validator>> {
        match self {
            Self::WithValidation { validators, .. } => Some(validators),
            Self::WithoutValidation { .. } => None,
        }
    }
}

/// Parsed by not yet validated
#[derive(Debug)]
pub struct RawGuard<Sanitizer, Validator> {
    pub sanitizers: Vec<Sanitizer>,
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeriveTrait {
    // Standard library
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    FromStr,
    AsRef,
    From,
    TryFrom,
    Into,
    Hash,
    Borrow,
    Display,
    Default,
    Deref,

    // External crates
    //
    #[cfg_attr(not(feature = "serde"), allow(dead_code))]
    SerdeSerialize,
    #[cfg_attr(not(feature = "serde"), allow(dead_code))]
    SerdeDeserialize,

    #[cfg_attr(not(feature = "schemars08"), allow(dead_code))]
    SchemarsJsonSchema,

    #[cfg_attr(not(feature = "arbitrary"), allow(dead_code))]
    ArbitraryArbitrary,

    #[cfg_attr(not(feature = "diesel-derive-newtype"), allow(dead_code))]
    DieselNewType,
}

pub type SpannedDeriveTrait = SpannedItem<DeriveTrait>;

pub trait TypeTrait {
    // If this is FromStr variant?
    fn is_from_str(&self) -> bool;
}

/// The flag the indicates that a newtype will be generated with extra constructor,
/// `::new_unchecked()` constructor which allows to avoid the guards.
/// Generally, usage of `new_unchecked` is discouraged.
#[derive(Debug, Default)]
pub enum NewUnchecked {
    #[default]
    Off,

    // `On` variant can be constructed when `new_unchecked` feature flag is enabled.
    #[allow(dead_code)]
    On,
}

pub struct GenerateParams<IT, Trait, Guard> {
    pub inner_type: IT,
    pub doc_attrs: Vec<Attribute>,
    pub traits: HashSet<Trait>,
    pub vis: syn::Visibility,
    pub type_name: TypeName,
    pub guard: Guard,
    pub new_unchecked: NewUnchecked,
    pub maybe_default_value: Option<syn::Expr>,
}

pub trait Newtype {
    type Sanitizer;
    type Validator;
    type TypedTrait;
    type InnerType;

    #[allow(clippy::type_complexity)]
    fn parse_attributes(
        attrs: TokenStream,
    ) -> Result<Attributes<Guard<Self::Sanitizer, Self::Validator>, SpannedDeriveTrait>, syn::Error>;

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedDeriveTrait>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error>;

    #[allow(clippy::type_complexity)]
    fn generate(
        params: GenerateParams<
            Self::InnerType,
            Self::TypedTrait,
            Guard<Self::Sanitizer, Self::Validator>,
        >,
    ) -> Result<TokenStream, syn::Error>;

    fn expand(
        typed_meta: TypedMeta,
        inner_type: Self::InnerType,
    ) -> Result<TokenStream, syn::Error> {
        let TypedMeta {
            doc_attrs,
            type_name,
            attrs,
            vis,
        } = typed_meta;
        let Attributes {
            guard,
            new_unchecked,
            default: maybe_default_value,
            derive_traits,
        } = Self::parse_attributes(attrs)?;
        let traits = Self::validate(&guard, derive_traits)?;
        let generated_output = Self::generate(GenerateParams {
            doc_attrs,
            traits,
            vis,
            type_name,
            guard,
            new_unchecked,
            maybe_default_value,
            inner_type,
        })?;
        Ok(generated_output)
    }
}

/// Represents a function that is used for custom sanitizers and validators specified
/// with `with =`.
/// It can be either pass to an existing function or a closure.
#[derive(Debug, Clone)]
pub enum CustomFunction {
    Path(Path),
    Closure(ExprClosure),
}

impl Parse for CustomFunction {
    fn parse(input: ParseStream) -> syn::Result<CustomFunction> {
        if let Ok(path) = input.parse::<Path>() {
            Ok(Self::Path(path))
        } else if let Ok(closure) = input.parse::<ExprClosure>() {
            Ok(Self::Closure(closure))
        } else {
            let msg = "Expected a path to function or a closure.";
            Err(syn::Error::new(input.span(), msg))
        }
    }
}

impl CustomFunction {
    pub fn try_into_typed(self, inner_type: &syn::Type) -> syn::Result<TypedCustomFunction> {
        match self {
            CustomFunction::Path(path) => Ok(TypedCustomFunction::Path(path)),
            CustomFunction::Closure(closure) => {
                // NOTE: this is a bit hacky, we're converting things to TokenStream and back.
                let input_token_stream = quote!(#closure);
                let output_token_stream = type_custom_closure(&input_token_stream, inner_type);
                let typed_closure: ExprClosure = syn::parse2(output_token_stream)?;
                Ok(TypedCustomFunction::Closure(typed_closure))
            }
        }
    }
}

impl ToTokens for CustomFunction {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        match self {
            CustomFunction::Path(path) => {
                path.to_tokens(token_stream);
            }
            CustomFunction::Closure(closure) => {
                closure.to_tokens(token_stream);
            }
        };
    }
}

/// Represents a function that is used for custom sanitizers and validators specified
/// with `with =`.
/// It's almost the same as CustomFunction with one important difference:
/// TypedCustomFunction is guaranteed to have arguments in closure to be typed.
/// While CustomFunction is used for parsing, TypedCustomFunction is used for code generation.
#[derive(Debug, Clone)]
pub enum TypedCustomFunction {
    Path(Path),
    Closure(ExprClosure),
}

impl ToTokens for TypedCustomFunction {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        match self {
            Self::Path(path) => {
                path.to_tokens(token_stream);
            }
            Self::Closure(closure) => {
                closure.to_tokens(token_stream);
            }
        };
    }
}

/// This trait allows to reuse validation of numeric validators.
pub trait NumericBoundValidator<T: Clone> {
    fn greater(&self) -> Option<T>;
    fn greater_or_equal(&self) -> Option<T>;
    fn less(&self) -> Option<T>;
    fn less_or_equal(&self) -> Option<T>;
}

macro_rules! impl_numeric_bound_validator {
    ($tp:ident) => {
        impl<T: Clone> crate::common::models::NumericBoundValidator<T> for $tp<T> {
            fn greater(&self) -> Option<T> {
                if let $tp::Greater(ValueOrExpr::Value(value)) = self {
                    Some(value.clone())
                } else {
                    None
                }
            }

            fn greater_or_equal(&self) -> Option<T> {
                if let $tp::GreaterOrEqual(ValueOrExpr::Value(value)) = self {
                    Some(value.clone())
                } else {
                    None
                }
            }

            fn less(&self) -> Option<T> {
                if let $tp::Less(ValueOrExpr::Value(value)) = self {
                    Some(value.clone())
                } else {
                    None
                }
            }

            fn less_or_equal(&self) -> Option<T> {
                if let $tp::LessOrEqual(ValueOrExpr::Value(value)) = self {
                    Some(value.clone())
                } else {
                    None
                }
            }
        }
    };
}

pub(crate) use impl_numeric_bound_validator;

/// The trait is used to generate tests for integer and float types, to ensure that
/// the upper boundary is not below the lower boundary.
pub trait NumericBound {
    fn upper(&self) -> Option<TokenStream>;
    fn lower(&self) -> Option<TokenStream>;
}

macro_rules! impl_numeric_bound_on_vec_of {
    ($validator:ident) => {
        impl<T: ::quote::ToTokens> crate::common::models::NumericBound for Vec<$validator<T>> {
            fn upper(&self) -> Option<TokenStream> {
                use ::quote::ToTokens;

                let values: Vec<TokenStream> = self
                    .iter()
                    .filter_map(|v| match v {
                        $validator::LessOrEqual(v) => Some(v),
                        $validator::Less(v) => Some(v),
                        _ => None,
                    })
                    .map(|v| v.to_token_stream())
                    .collect();

                if values.len() > 1 {
                    // This should actually never happened, since there are validation in place that
                    // prevents usage of `less_or_equal` and `less` at the same time,
                    // but we want to be sure.
                    panic!("It's not allowed to use less_or_equal and less validators at the same time");
                }

                values.into_iter().next()
            }

            fn lower(&self) -> Option<TokenStream> {
                use ::quote::ToTokens;

                let values: Vec<TokenStream> = self
                    .iter()
                    .filter_map(|v| match v {
                        $validator::GreaterOrEqual(v) => Some(v),
                        $validator::Greater(v) => Some(v),
                        _ => None,
                    })
                    .map(|v| v.to_token_stream())
                    .collect();

                if values.len() > 1 {
                    // This should actually never happened, since there are validation in place that
                    // prevents usage of `greater_or_equal` and `greater` at the same time,
                    // but we want to be sure.
                    panic!(
                        "It's not allowed to use greater_or_equal and greater validators at the same time"
                    );
                }

                values.into_iter().next()
            }
        }
    }
}

pub(crate) use impl_numeric_bound_on_vec_of;
