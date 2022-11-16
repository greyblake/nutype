use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, DeriveInput};

use crate::{
    common::parse::is_doc_attribute,
    models::{InnerType, TypeNameAndInnerType},
};

pub fn parse_type_name_and_inner_type(
    token_stream: TokenStream,
) -> Result<TypeNameAndInnerType, syn::Error> {
    let input: DeriveInput = syn::parse(token_stream.into()).unwrap();

    let input_span = input.span();
    let DeriveInput {
        attrs,
        data,
        vis,
        ident: type_name,
        generics: _,
    } = input;

    let doc_attrs: Vec<Attribute> = attrs.into_iter().filter(is_doc_attribute).collect();

    let data_struct = match &data {
        syn::Data::Struct(v) => v.clone(),
        _ => {
            let error =
                syn::Error::new(input_span, "#[nutype] can be used only with tuple structs.");
            return Err(error);
        }
    };

    let fields_unnamed = match data_struct.fields {
        syn::Fields::Unnamed(fu) => fu,
        _ => {
            let error =
                syn::Error::new(input_span, "#[nutype] can be used only with tuple structs.");
            return Err(error);
        }
    };

    let seg = fields_unnamed.unnamed.iter().next().unwrap();

    let type_path = match seg.ty.clone() {
        syn::Type::Path(tp) => tp,
        _ => {
            let error = syn::Error::new(
                seg.span(),
                "#[nutype] requires a simple inner type (e.g. String, i32, etc.)",
            );
            return Err(error);
        }
    };

    let type_path_str = type_path.into_token_stream().to_string();

    let inner_type = match type_path_str.as_ref() {
        "String" => InnerType::String,
        "u8" => InnerType::Number(crate::models::NumberType::U8),
        "u16" => InnerType::Number(crate::models::NumberType::U16),
        "u32" => InnerType::Number(crate::models::NumberType::U32),
        "u64" => InnerType::Number(crate::models::NumberType::U64),
        "u128" => InnerType::Number(crate::models::NumberType::U128),
        "usize" => InnerType::Number(crate::models::NumberType::Usize),
        "i8" => InnerType::Number(crate::models::NumberType::I8),
        "i16" => InnerType::Number(crate::models::NumberType::I16),
        "i32" => InnerType::Number(crate::models::NumberType::I32),
        "i64" => InnerType::Number(crate::models::NumberType::I64),
        "i128" => InnerType::Number(crate::models::NumberType::I128),
        "isize" => InnerType::Number(crate::models::NumberType::Isize),
        "f32" => InnerType::Number(crate::models::NumberType::F32),
        "f64" => InnerType::Number(crate::models::NumberType::F64),
        tp => {
            let error = syn::Error::new(
                seg.span(),
                format!("#[nutype] does not support `{tp}` as inner type"),
            );
            return Err(error);
        }
    };

    Ok(TypeNameAndInnerType {
        doc_attrs,
        type_name,
        inner_type,
        vis,
    })
}
