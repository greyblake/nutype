mod gen;
mod models;
mod parser;

use gen::gen_nutype_for_string;
use models::{InnerType, TypeNameAndInnerType};
use parser::{parse_attributes, parse_type_name_and_inner_type};
use proc_macro2::TokenStream;

#[proc_macro_attribute]
pub fn nutype(
    attrs: proc_macro::TokenStream,
    type_definition: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_nutype(attrs.into(), type_definition.into()).into()
}

fn expand_nutype(attrs: TokenStream, type_definition: TokenStream) -> TokenStream {
    let TypeNameAndInnerType {
        type_name,
        inner_type,
    } = parse_type_name_and_inner_type(type_definition);

    match inner_type {
        InnerType::String => {
            // TODO: rename to parse_string_attributes
            let meta = parse_attributes(attrs);
            gen_nutype_for_string(&type_name, meta)
        }
    }
}
