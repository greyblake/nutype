use std::fmt::Debug;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::Attribute;

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

impl<T> Spanned for SpannedItem<T> {
    fn span(&self) -> Span {
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
    Integer(IntegerType),
    Float(FloatType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    Isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatType {
    F32,
    F64,
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

impl ToTokens for IntegerType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let type_stream = match self {
            Self::U8 => quote!(u8),
            Self::U16 => quote!(u16),
            Self::U32 => quote!(u32),
            Self::U64 => quote!(u64),
            Self::U128 => quote!(u128),
            Self::Usize => quote!(usize),
            Self::I8 => quote!(i8),
            Self::I16 => quote!(i16),
            Self::I32 => quote!(i32),
            Self::I64 => quote!(i64),
            Self::I128 => quote!(i128),
            Self::Isize => quote!(isize),
        };
        type_stream.to_tokens(token_stream);
    }
}

impl ToTokens for FloatType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        let type_stream = match self {
            Self::F32 => quote!(f32),
            Self::F64 => quote!(f64),
        };
        type_stream.to_tokens(token_stream);
    }
}

#[derive(Debug)]
pub struct TypeName(Ident);

impl TypeName {
    pub fn new(name: Ident) -> Self {
        Self(name)
    }
}

impl core::fmt::Display for TypeName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToTokens for TypeName {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        self.0.to_tokens(token_stream)
    }
}

#[derive(Debug)]
pub struct ErrorTypeName(Ident);

impl ErrorTypeName {
    pub fn new(name: Ident) -> Self {
        Self(name)
    }
}

impl core::fmt::Display for ErrorTypeName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToTokens for ErrorTypeName {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        self.0.to_tokens(token_stream)
    }
}

#[derive(Debug)]
pub struct NewtypeMeta {
    pub type_name: TypeName,
    pub inner_type: InnerType,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    // External crates
    //
    #[cfg_attr(not(feature = "serde1"), allow(dead_code))]
    SerdeSerialize,

    #[cfg_attr(not(feature = "serde1"), allow(dead_code))]
    SerdeDeserialize,
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
