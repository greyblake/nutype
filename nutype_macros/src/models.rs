use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Attribute;

use crate::base::SpannedItem;
pub use crate::string::models::{StringSanitizer, StringValidator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName(String);

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

// TODO: Rename
#[derive(Debug)]
pub struct TypeNameAndInnerType {
    pub type_name: Ident,
    pub inner_type: InnerType,
    pub vis: syn::Visibility,
    pub doc_attrs: Vec<Attribute>,
    pub derive_traits: Vec<SpannedDeriveTrait>,
}

/// Validated model, that represents precisly what needs to be generated.
#[derive(Debug)]
pub enum NewtypeMeta<Sanitizer, Validator> {
    // TODO: rename variants
    From {
        sanitizers: Vec<Sanitizer>,
    },
    TryFrom {
        sanitizers: Vec<Sanitizer>,
        validators: Vec<Validator>,
    },
}

impl<Sanitizer, Validator> NewtypeMeta<Sanitizer, Validator> {
    pub fn has_validation(&self) -> bool {
        match self {
            Self::From { .. } => false,
            Self::TryFrom { .. } => true,
        }
    }
}

/// Parsed by not yet validated
#[derive(Debug)]
pub struct RawNewtypeMeta<Sanitizer, Validator> {
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
    // Serialize,
    // Deserialize,
    // Arbitrary,
}

pub type SpannedDeriveTrait = SpannedItem<DeriveTrait>;
