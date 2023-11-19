//! Macro implementations for `nutype` crate.
//!
//! Don't use this crate directly, use `nutype` instead.
//!
//! For more information please refer to [nutype](https://docs.rs/nutype) documentation.

mod any;
mod common;
mod float;
mod integer;
mod string;

use any::AnyNewtype;
use common::{
    models::{InnerType, Newtype, TypedMeta},
    parse::meta::parse_meta,
};
use float::{models::FloatInnerType, FloatNewtype};
use integer::{models::IntegerInnerType, IntegerNewtype};
use proc_macro2::TokenStream;
use string::StringNewtype;

/// Defines sanitizers and validators on a newtype.
/// Guarantees that the type can be instantiated only with valid values.
/// See the documentation for [nutype](https://docs.rs/nutype) crate for more information.
#[proc_macro_attribute]
pub fn nutype(
    attrs: proc_macro::TokenStream,
    type_definition: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_nutype(attrs.into(), type_definition.into())
        .unwrap_or_else(|e| syn::Error::to_compile_error(&e))
        .into()
}

fn expand_nutype(
    attrs: TokenStream,
    type_definition: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let meta = parse_meta(type_definition)?;
    let (typed_meta, inner_type) = meta.into_typed_meta(attrs);

    match inner_type {
        InnerType::String(tp) => StringNewtype::expand(typed_meta, tp),
        InnerType::Integer(inner) => expand_nutype_integer(typed_meta, inner),
        InnerType::Float(inner) => expand_nutype_float(typed_meta, inner),
        InnerType::Any(any_inner_type) => AnyNewtype::expand(typed_meta, any_inner_type),
    }
}

fn expand_nutype_integer(
    typed_meta: TypedMeta,
    inner: IntegerInnerType,
) -> Result<TokenStream, syn::Error> {
    use IntegerInnerType::*;

    match inner {
        U8 => IntegerNewtype::<u8>::expand(typed_meta, U8),
        U16 => IntegerNewtype::<u16>::expand(typed_meta, U16),
        U32 => IntegerNewtype::<u32>::expand(typed_meta, U32),
        U64 => IntegerNewtype::<u64>::expand(typed_meta, U64),
        U128 => IntegerNewtype::<u128>::expand(typed_meta, U128),
        Usize => IntegerNewtype::<usize>::expand(typed_meta, Usize),
        I8 => IntegerNewtype::<i8>::expand(typed_meta, I8),
        I16 => IntegerNewtype::<i16>::expand(typed_meta, I16),
        I32 => IntegerNewtype::<i32>::expand(typed_meta, I32),
        I64 => IntegerNewtype::<i64>::expand(typed_meta, I64),
        I128 => IntegerNewtype::<i128>::expand(typed_meta, I128),
        Isize => IntegerNewtype::<isize>::expand(typed_meta, Isize),
    }
}

fn expand_nutype_float(
    typed_meta: TypedMeta,
    inner: FloatInnerType,
) -> Result<TokenStream, syn::Error> {
    use FloatInnerType::*;

    match inner {
        F32 => FloatNewtype::<f32>::expand(typed_meta, F32),
        F64 => FloatNewtype::<f64>::expand(typed_meta, F64),
    }
}
