use std::collections::HashSet;
use std::fmt::Debug;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Attribute;

use crate::float::models::FloatInnerType;
use crate::integer::models::IntegerInnerType;

/// A trait that may be implemented by enums with payload.
/// It's mostly used to detect duplicates of validators and sanitizers regardless of their payload.
pub trait Kind {
    type Kind: PartialEq + Eq + Debug + Clone + Copy;

    fn kind(&self) -> Self::Kind;
}

/// A spanned item. An item can be anything that cares a domain value.
/// Keeping a span allows to throw good precise error messages at the validation stage.
#[derive(Debug)]
pub struct SpannedItem<T> {
    pub item: T,
    pub span: Span,
}

impl<T> SpannedItem<T> {
    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T: Kind> Kind for SpannedItem<T> {
    type Kind = <T as Kind>::Kind;

    fn kind(&self) -> Self::Kind {
        self.item.kind()
    }
}

/// Represents the inner type of a newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerType {
    String,
    Integer(IntegerInnerType),
    Float(FloatInnerType),
}

impl From<IntegerInnerType> for InnerType {
    fn from(tp: IntegerInnerType) -> InnerType {
        InnerType::Integer(tp)
    }
}

impl From<FloatInnerType> for InnerType {
    fn from(tp: FloatInnerType) -> InnerType {
        InnerType::Float(tp)
    }
}

impl ToTokens for InnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        match self {
            InnerType::String => {
                quote!(String).to_tokens(token_stream);
            }
            InnerType::Integer(integer_type) => {
                integer_type.to_tokens(token_stream);
            }
            InnerType::Float(float_type) => {
                float_type.to_tokens(token_stream);
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

// Repesents a type for a validation error.
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
    pub derive_traits: Vec<SpannedDeriveTrait>,
}

impl Meta {
    pub fn into_typed_meta(self, attrs: TokenStream) -> (TypedMeta, InnerType) {
        let Self {
            doc_attrs,
            type_name,
            inner_type,
            vis,
            derive_traits,
        } = self;
        let typed_meta = TypedMeta {
            doc_attrs,
            type_name,
            attrs,
            vis,
            derive_traits,
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
    pub derive_traits: Vec<SpannedDeriveTrait>,
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
pub struct Attributes<G> {
    /// Guard contains sanitizers and validators
    pub guard: G,

    /// `new_unchecked` flag
    pub new_unchecked: NewUnchecked,

    /// Value for Default trait. Provide with `default = `
    pub maybe_default_value: Option<TokenStream>,
}

impl<Sanitizer, Validator> Guard<Sanitizer, Validator> {
    pub fn has_validation(&self) -> bool {
        match self {
            Self::WithoutValidation { .. } => false,
            Self::WithValidation { .. } => true,
        }
    }
}

/// Parsed by not yet validated
#[derive(Debug)]
pub struct RawGuard<Sanitizer, Validator> {
    pub sanitizers: Vec<Sanitizer>,
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveTrait {
    Asterisk,
    Normal(NormalDeriveTrait),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalDeriveTrait {
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
}

pub type SpannedDeriveTrait = SpannedItem<DeriveTrait>;

/// The flag the indicates that a newtype will be generated with extra constructor,
/// `::new_unchecked()` constructor which allows to avoid the guards.
/// Generally, usage of `new_unchecked` is discouraged.
#[derive(Debug)]
pub enum NewUnchecked {
    // `On` variant can be constructed when `new_unchecked` feature flag is enabled.
    #[allow(dead_code)]
    On,
    Off,
}

pub struct GenerateParams<T, G> {
    pub doc_attrs: Vec<Attribute>,
    pub traits: HashSet<T>,
    pub vis: syn::Visibility,
    pub type_name: TypeName,
    pub guard: G,
    pub new_unchecked: NewUnchecked,
    pub maybe_default_value: Option<TokenStream>,
}

pub trait Newtype {
    type Sanitizer;
    type Validator;
    type TypedTrait;

    #[allow(clippy::type_complexity)]
    fn parse_attributes(
        attrs: TokenStream,
    ) -> Result<Attributes<Guard<Self::Sanitizer, Self::Validator>>, syn::Error>;

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedItem<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error>;

    fn generate(
        params: GenerateParams<Self::TypedTrait, Guard<Self::Sanitizer, Self::Validator>>,
    ) -> TokenStream;

    fn expand(typed_meta: TypedMeta) -> Result<TokenStream, syn::Error> {
        let TypedMeta {
            doc_attrs,
            type_name,
            attrs,
            vis,
            derive_traits,
        } = typed_meta;
        let Attributes {
            guard,
            new_unchecked,
            maybe_default_value,
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
        });
        Ok(generated_output)
    }
}
