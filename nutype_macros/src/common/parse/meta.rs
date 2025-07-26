use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, DeriveInput, Visibility, spanned::Spanned};

use crate::{
    any::models::AnyInnerType,
    common::{
        models::{InnerType, Meta, TypeName},
        parse::{intercept_derive_macro, is_derive_attribute, is_doc_attribute},
    },
    float::models::FloatInnerType,
    integer::models::IntegerInnerType,
    string::models::StringInnerType,
};

pub fn parse_meta(token_stream: TokenStream) -> Result<Meta, syn::Error> {
    let input: DeriveInput = syn::parse(token_stream.into())?;

    let input_span = input.span();
    let DeriveInput {
        attrs,
        data,
        vis,
        ident: type_name,
        generics,
    } = input;

    let type_name = TypeName::new(type_name);

    validate_supported_attrs(&attrs)?;

    intercept_derive_macro(&attrs)?;
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

    let seg = fields_unnamed.unnamed.iter().next().ok_or_else(|| {
        let suggested_struct = quote::quote!(
            #vis struct #type_name(i32)
        )
        .to_string();
        let msg = format!(
            "Your wish to use #[nutype] with an empty tuple struct is respected.\n\
             But how about NO?\n\
             I bet you'll be luckier trying out something like this:\n\n\
             {suggested_struct};\n\n"
        );
        syn::Error::new(fields_unnamed.span(), msg)
    })?;
    validate_inner_field_visibility(&seg.vis)?;

    let type_path_str = seg.ty.clone().into_token_stream().to_string();

    let inner_type = match type_path_str.as_ref() {
        "String" => InnerType::String(StringInnerType),
        "u8" => InnerType::Integer(IntegerInnerType::U8),
        "u16" => InnerType::Integer(IntegerInnerType::U16),
        "u32" => InnerType::Integer(IntegerInnerType::U32),
        "u64" => InnerType::Integer(IntegerInnerType::U64),
        "u128" => InnerType::Integer(IntegerInnerType::U128),
        "usize" => InnerType::Integer(IntegerInnerType::Usize),
        "i8" => InnerType::Integer(IntegerInnerType::I8),
        "i16" => InnerType::Integer(IntegerInnerType::I16),
        "i32" => InnerType::Integer(IntegerInnerType::I32),
        "i64" => InnerType::Integer(IntegerInnerType::I64),
        "i128" => InnerType::Integer(IntegerInnerType::I128),
        "isize" => InnerType::Integer(IntegerInnerType::Isize),
        "f32" => InnerType::Float(FloatInnerType::F32),
        "f64" => InnerType::Float(FloatInnerType::F64),
        _ => InnerType::Any(AnyInnerType::new(seg.clone())),
    };

    Ok(Meta {
        doc_attrs,
        type_name,
        generics,
        inner_type,
        vis,
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
                "#[nutype] does not support this attribute.",
            ));
        }
    }

    Ok(())
}

fn validate_inner_field_visibility(vis: &Visibility) -> Result<(), syn::Error> {
    match vis {
        Visibility::Inherited => Ok(()),
        Visibility::Public(_) | Visibility::Restricted(_) => {
            let msg = "Oh, setting visibility for the inner field is forbidden by #[nutype].\nThe whole point is to guarantee that no value can be created without passing the guards (sanitizers and validators).\nWe do hope for your understanding and wish you a good sunny day!";
            Err(syn::Error::new(vis.span(), msg))
        }
    }
}
