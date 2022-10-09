use proc_macro2::{TokenStream as TokenStream2};
use quote::ToTokens;
use syn::DeriveInput;

use crate::models::{TypeNameAndInnerType, InnerType};

// TODO: Parse visibility as well
pub fn parse_type_name_and_inner_type(token_stream: TokenStream2) -> TypeNameAndInnerType {
    let input: DeriveInput = syn::parse(token_stream.into()).unwrap();

    let type_name = input.ident;


    let data_struct = match &input.data {
        syn::Data::Struct(v) => v.clone(),
        _ => panic!("Expected syn::Data::Struct, got: {:?}", input.data)
    };

    let fields_unnamed = match data_struct.fields {
        syn::Fields::Unnamed(fu) => fu,
        _ => panic!("Expected syn::Fields::Unnamed, got: {:?}", &data_struct.fields)
    };

    let seg = fields_unnamed.unnamed.iter().next().unwrap();

    let type_path = match seg.ty.clone() {
        syn::Type::Path(tp) => tp,
        _ => panic!("Expected syn::Type::Path, got: {:?}", &seg.ty)
    };

    let type_path_str = type_path.into_token_stream().to_string();

    let inner_type = match type_path_str.as_ref() {
        "String" => InnerType::String,
        tp => panic!("Unsupported inner type: {}", tp),
    };

    TypeNameAndInnerType {
        type_name,
        inner_type
    }
}
