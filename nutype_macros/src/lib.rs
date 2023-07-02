mod common;
mod float;
mod integer;
mod string;
mod utils;

use common::models::{InnerType, Newtype};
use common::parse::meta::parse_meta;
use float::{models::FloatInnerType, FloatNewtype};
use integer::models::IntegerInnerType;
use integer::IntegerNewtype;
use proc_macro2::TokenStream;
use string::StringNewtype;

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
    use FloatInnerType::*;
    use IntegerInnerType::*;

    let meta = parse_meta(type_definition)?;
    let (typed_meta, inner_type) = meta.to_typed_meta(attrs);

    match inner_type {
        InnerType::String => StringNewtype::expand(typed_meta),
        InnerType::Integer(U8) => IntegerNewtype::<u8>::expand(typed_meta),
        InnerType::Integer(U16) => IntegerNewtype::<u16>::expand(typed_meta),
        InnerType::Integer(U32) => IntegerNewtype::<u32>::expand(typed_meta),
        InnerType::Integer(U64) => IntegerNewtype::<u64>::expand(typed_meta),
        InnerType::Integer(U128) => IntegerNewtype::<u128>::expand(typed_meta),
        InnerType::Integer(Usize) => IntegerNewtype::<usize>::expand(typed_meta),
        InnerType::Integer(I8) => IntegerNewtype::<i8>::expand(typed_meta),
        InnerType::Integer(I16) => IntegerNewtype::<i16>::expand(typed_meta),
        InnerType::Integer(I32) => IntegerNewtype::<i32>::expand(typed_meta),
        InnerType::Integer(I64) => IntegerNewtype::<i64>::expand(typed_meta),
        InnerType::Integer(I128) => IntegerNewtype::<i128>::expand(typed_meta),
        InnerType::Integer(Isize) => IntegerNewtype::<isize>::expand(typed_meta),
        InnerType::Float(F32) => FloatNewtype::<f32>::expand(typed_meta),
        InnerType::Float(F64) => FloatNewtype::<f64>::expand(typed_meta),
    }
}
