mod common;
mod models;
mod number;
mod parse;
mod string;

use models::{InnerType, TypeNameAndInnerType};
use parse::parse_type_name_and_inner_type;
use proc_macro2::TokenStream;
use quote::quote;
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
    }
}
