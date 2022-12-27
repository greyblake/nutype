mod common;
mod float;
mod integer;
mod models;
mod string;

use std::{fmt::Debug, str::FromStr};

use common::models::{FloatType, InnerType, IntegerType, NewtypeMeta, SpannedDeriveTrait};
use common::parse::meta::parse_meta;
use float::validate::validate_float_derive_traits;
use integer::validate::validate_integer_derive_traits;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use string::{gen::gen_nutype_for_string, validate::validate_string_derive_traits};
use syn::Visibility;

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
        InnerType::String => {
            let meta = string::parse::parse_attributes(attrs)?;
            let traits = validate_string_derive_traits(&meta, derive_traits)?;
            Ok(gen_nutype_for_string(
                doc_attrs, traits, vis, &type_name, meta,
            ))
        }
        InnerType::Integer(tp) => {
            let params = NumberParams {
                doc_attrs,
                vis,
                tp,
                type_name,
                attrs,
                derive_traits,
            };
            match tp {
                IntegerType::U8 => parse_integer_attrs_and_gen::<u8>(params),
                IntegerType::U16 => parse_integer_attrs_and_gen::<u16>(params),
                IntegerType::U32 => parse_integer_attrs_and_gen::<u32>(params),
                IntegerType::U64 => parse_integer_attrs_and_gen::<u64>(params),
                IntegerType::U128 => parse_integer_attrs_and_gen::<u128>(params),
                IntegerType::Usize => parse_integer_attrs_and_gen::<usize>(params),
                IntegerType::I8 => parse_integer_attrs_and_gen::<i8>(params),
                IntegerType::I16 => parse_integer_attrs_and_gen::<i16>(params),
                IntegerType::I32 => parse_integer_attrs_and_gen::<i32>(params),
                IntegerType::I64 => parse_integer_attrs_and_gen::<i64>(params),
                IntegerType::I128 => parse_integer_attrs_and_gen::<i128>(params),
                IntegerType::Isize => parse_integer_attrs_and_gen::<isize>(params),
            }
        }
        InnerType::Float(tp) => {
            let params = NumberParams {
                doc_attrs,
                vis,
                tp,
                type_name,
                attrs,
                derive_traits,
            };
            match tp {
                FloatType::F32 => parse_float_attrs_and_gen::<f32>(params),
                FloatType::F64 => parse_float_attrs_and_gen::<f64>(params),
            }
        }
    }
}

struct NumberParams<NumberType> {
    doc_attrs: Vec<syn::Attribute>,
    vis: Visibility,
    tp: NumberType,
    type_name: Ident,
    attrs: TokenStream,
    derive_traits: Vec<SpannedDeriveTrait>,
}

fn parse_integer_attrs_and_gen<T>(
    params: NumberParams<IntegerType>,
) -> Result<TokenStream, syn::Error>
where
    T: FromStr + ToTokens + PartialOrd + Clone,
    <T as FromStr>::Err: Debug,
{
    let NumberParams {
        doc_attrs,
        vis,
        tp,
        type_name,
        attrs,
        derive_traits,
    } = params;
    let meta = integer::parse::parse_attributes::<T>(attrs)?;
    let traits = validate_integer_derive_traits(derive_traits, meta.has_validation())?;
    Ok(integer::gen::gen_nutype_for_integer(
        doc_attrs, vis, tp, &type_name, meta, traits,
    ))
}

fn parse_float_attrs_and_gen<T>(params: NumberParams<FloatType>) -> Result<TokenStream, syn::Error>
where
    T: FromStr + ToTokens + PartialOrd + Clone,
    <T as FromStr>::Err: Debug,
{
    let NumberParams {
        doc_attrs,
        vis,
        tp,
        type_name,
        attrs,
        derive_traits,
    } = params;
    let meta = float::parse::parse_attributes::<T>(attrs)?;
    let traits = validate_float_derive_traits(derive_traits, meta.has_validation())?;
    Ok(float::gen::gen_nutype_for_float(
        doc_attrs, vis, tp, &type_name, meta, traits,
    ))
}
