mod common;
mod float;
mod integer;
mod string;

use common::{
    models::{InnerType, Newtype},
    parse::meta::parse_meta,
};
use float::{models::FloatInnerType, FloatNewtype};
use integer::{models::IntegerInnerType, IntegerNewtype};
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
    let (typed_meta, inner_type) = meta.into_typed_meta(attrs);

    match inner_type {
        InnerType::String(tp) => StringNewtype::expand(typed_meta, tp),
        InnerType::Integer(U8) => IntegerNewtype::<u8>::expand(typed_meta, U8),
        InnerType::Integer(U16) => IntegerNewtype::<u16>::expand(typed_meta, U16),
        InnerType::Integer(U32) => IntegerNewtype::<u32>::expand(typed_meta, U32),
        InnerType::Integer(U64) => IntegerNewtype::<u64>::expand(typed_meta, U64),
        InnerType::Integer(U128) => IntegerNewtype::<u128>::expand(typed_meta, U128),
        InnerType::Integer(Usize) => IntegerNewtype::<usize>::expand(typed_meta, Usize),
        InnerType::Integer(I8) => IntegerNewtype::<i8>::expand(typed_meta, I8),
        InnerType::Integer(I16) => IntegerNewtype::<i16>::expand(typed_meta, I16),
        InnerType::Integer(I32) => IntegerNewtype::<i32>::expand(typed_meta, I32),
        InnerType::Integer(I64) => IntegerNewtype::<i64>::expand(typed_meta, I64),
        InnerType::Integer(I128) => IntegerNewtype::<i128>::expand(typed_meta, I128),
        InnerType::Integer(Isize) => IntegerNewtype::<isize>::expand(typed_meta, Isize),
        InnerType::Float(F32) => FloatNewtype::<f32>::expand(typed_meta, F32),
        InnerType::Float(F64) => FloatNewtype::<f64>::expand(typed_meta, F64),
    }
}
