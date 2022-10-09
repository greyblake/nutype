mod models;
mod parser;
mod gen;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;


use models::{StringSanitizer, StringValidator, TypeNameAndInnerType, NewtypeStringMeta};
use parser::parse_type_name_and_inner_type;
use gen::{
    gen_module_name_for_type,
    gen_error_type_name,
    gen_string_implementation,
};

#[proc_macro_attribute]
pub fn nutype(attrs: TokenStream, type_definition: TokenStream) -> TokenStream {
    inner_nutype(attrs.into(), type_definition.into()).into()
}


fn inner_nutype(attrs: TokenStream2, type_definition: TokenStream2) -> TokenStream2 {
    let TypeNameAndInnerType { type_name, inner_type } = parse_type_name_and_inner_type(type_definition);
    let module_name = gen_module_name_for_type(&type_name);

    let sanitizers = vec![
        StringSanitizer::Trim,
        StringSanitizer::Lowecase,
    ];

    let validators = vec![
        StringValidator::MaxLen(255),
        StringValidator::MinLen(6),
    ];

    let meta = NewtypeStringMeta { sanitizers, validators };


    // TODO: refactor!
    let error_type_import = if meta.validators.is_empty() {
        quote!()
    } else {
        let error_type_name = gen_error_type_name(&type_name);
        quote! (
            pub use #module_name::#error_type_name;
        )
    };

    let implementation = gen_string_implementation(&type_name, inner_type, &meta);

    // TODO: respect visiblity!
    quote!(
        mod #module_name {
            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub struct #type_name(#inner_type);

            #implementation
        }
        pub use #module_name::#type_name;
        #error_type_import
    )
}

