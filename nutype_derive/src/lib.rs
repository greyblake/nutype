mod base;
mod common;
mod models;
mod number;
mod parse;
mod string;

use std::{fmt::Debug, str::FromStr};

use models::{InnerType, NumberType, TypeNameAndInnerType};
use number::gen::gen_nutype_for_number;
use parse::parse_type_name_and_inner_type;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use string::gen::gen_nutype_for_string;

#[proc_macro_attribute]
pub fn nutype(
    attrs: proc_macro::TokenStream,
    type_definition: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_nutype(attrs.into(), type_definition.into())
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}

fn expand_nutype(
    attrs: TokenStream,
    type_definition: TokenStream,
) -> Result<TokenStream, Vec<syn::Error>> {
    let TypeNameAndInnerType {
        type_name,
        inner_type,
    } = parse_type_name_and_inner_type(type_definition)?;

    match inner_type {
        InnerType::String => {
            // TODO: rename to parse_string_attributes
            let meta = string::parse::parse_attributes(attrs)?;
            Ok(gen_nutype_for_string(&type_name, meta))
        }
        InnerType::Number(tp) => match tp {
            NumberType::U8 => parse_number_attrs_and_gen::<u8>(tp, &type_name, attrs),
            NumberType::U16 => parse_number_attrs_and_gen::<u16>(tp, &type_name, attrs),
            NumberType::U32 => parse_number_attrs_and_gen::<u32>(tp, &type_name, attrs),
            NumberType::U64 => parse_number_attrs_and_gen::<u64>(tp, &type_name, attrs),
            NumberType::U128 => parse_number_attrs_and_gen::<u128>(tp, &type_name, attrs),
            NumberType::Usize => parse_number_attrs_and_gen::<usize>(tp, &type_name, attrs),
            NumberType::I8 => parse_number_attrs_and_gen::<i8>(tp, &type_name, attrs),
            NumberType::I16 => parse_number_attrs_and_gen::<i16>(tp, &type_name, attrs),
            NumberType::I32 => parse_number_attrs_and_gen::<i32>(tp, &type_name, attrs),
            NumberType::I64 => parse_number_attrs_and_gen::<i64>(tp, &type_name, attrs),
            NumberType::I128 => parse_number_attrs_and_gen::<i128>(tp, &type_name, attrs),
            NumberType::Isize => parse_number_attrs_and_gen::<isize>(tp, &type_name, attrs),
        },
    }
}

fn parse_number_attrs_and_gen<T>(
    tp: NumberType,
    type_name: &Ident,
    attrs: TokenStream,
) -> Result<TokenStream, Vec<syn::Error>>
where
    T: FromStr + ToTokens + Ord,
    <T as FromStr>::Err: Debug,
{
    let meta = number::parse::parse_attributes::<T>(attrs)?;
    Ok(gen_nutype_for_number(tp, type_name, meta))
}
