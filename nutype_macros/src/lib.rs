mod common;
mod float;
mod integer;
mod string;
mod utils;

use common::models::{InnerType, Newtype, NewtypeMeta};
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
    let NewtypeMeta {
        doc_attrs,
        type_name,
        inner_type,
        vis,
        derive_traits,
    } = parse_meta(type_definition)?;
    match inner_type {
        InnerType::String => StringNewtype::expand(attrs, doc_attrs, type_name, vis, derive_traits),
        InnerType::Integer(tp) => {
            // TODO: Oh oh.. Try to DRY the repetition
            match tp {
                IntegerInnerType::U8 => {
                    IntegerNewtype::<u8>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::U16 => {
                    IntegerNewtype::<u16>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::U32 => {
                    IntegerNewtype::<u32>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::U64 => {
                    IntegerNewtype::<u64>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::U128 => {
                    IntegerNewtype::<u128>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::Usize => {
                    IntegerNewtype::<usize>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::I8 => {
                    IntegerNewtype::<i8>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::I16 => {
                    IntegerNewtype::<i16>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::I32 => {
                    IntegerNewtype::<i32>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::I64 => {
                    IntegerNewtype::<i64>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::I128 => {
                    IntegerNewtype::<i128>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
                IntegerInnerType::Isize => {
                    IntegerNewtype::<isize>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
                }
            }
        }
        InnerType::Float(tp) => match tp {
            FloatInnerType::F32 => {
                FloatNewtype::<f32>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
            }
            FloatInnerType::F64 => {
                FloatNewtype::<f64>::expand(attrs, doc_attrs, type_name, vis, derive_traits)
            }
        },
    }
}
