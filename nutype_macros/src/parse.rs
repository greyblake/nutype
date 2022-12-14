use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, DeriveInput, Visibility};

use crate::{
    common::parse::{is_derive_attribute, is_doc_attribute, parse_derive_traits},
    models::{InnerType, NewtypeMeta},
};

pub fn parse_meta(token_stream: TokenStream) -> Result<NewtypeMeta, syn::Error> {
    let input: DeriveInput = syn::parse(token_stream.into()).unwrap();

    let input_span = input.span();
    let DeriveInput {
        attrs,
        data,
        vis,
        ident: type_name,
        generics: _,
    } = input;

    validate_supported_attrs(&attrs)?;

    let derive_traits = parse_derive_traits(&attrs)?;
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

    validate_inner_field_visibility(&seg.vis)?;

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
        "u8" => InnerType::Integer(crate::models::IntegerType::U8),
        "u16" => InnerType::Integer(crate::models::IntegerType::U16),
        "u32" => InnerType::Integer(crate::models::IntegerType::U32),
        "u64" => InnerType::Integer(crate::models::IntegerType::U64),
        "u128" => InnerType::Integer(crate::models::IntegerType::U128),
        "usize" => InnerType::Integer(crate::models::IntegerType::Usize),
        "i8" => InnerType::Integer(crate::models::IntegerType::I8),
        "i16" => InnerType::Integer(crate::models::IntegerType::I16),
        "i32" => InnerType::Integer(crate::models::IntegerType::I32),
        "i64" => InnerType::Integer(crate::models::IntegerType::I64),
        "i128" => InnerType::Integer(crate::models::IntegerType::I128),
        "isize" => InnerType::Integer(crate::models::IntegerType::Isize),
        "f32" => InnerType::Float(crate::models::FloatType::F32),
        "f64" => InnerType::Float(crate::models::FloatType::F64),
        tp => {
            let error = syn::Error::new(
                seg.span(),
                format!("#[nutype] does not support `{tp}` as inner type"),
            );
            return Err(error);
        }
    };

    Ok(NewtypeMeta {
        doc_attrs,
        type_name,
        inner_type,
        vis,
        derive_traits,
    })
}

fn validate_supported_attrs(attrs: &[syn::Attribute]) -> Result<(), syn::Error> {
    fn is_supported_attr(attr: &syn::Attribute) -> bool {
        is_doc_attribute(attr) || is_derive_attribute(attr)
    }

    for attr in attrs {
        if !is_supported_attr(attr) {
            return Err(syn::Error::new(
                attr.span(),
                "#[nutype] does not support this attribute",
            ));
        }
    }

    Ok(())
}

fn validate_inner_field_visibility(vis: &Visibility) -> Result<(), syn::Error> {
    match vis {
        Visibility::Inherited => Ok(()),
        Visibility::Public(_) | Visibility::Crate(_) | Visibility::Restricted(_) => {
            let msg = "Oh, setting visibility for the inner field is forbidden by #[nutype].\nThe whole point is to guarantee that no value can be created without passing the guards (sanitizers and validators).\nI hope for your understanding and wishing you a good sunny day!";
            Err(syn::Error::new(vis.span(), msg))
        }
    }
}
