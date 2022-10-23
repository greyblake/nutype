mod gen;
mod models;
mod parser;

use proc_macro2::TokenStream;
use quote::quote;

use gen::{gen_error_type_name, gen_module_name_for_type, gen_string_implementation};
use models::{TypeNameAndInnerType, InnerType};
use parser::{parse_attributes, parse_type_name_and_inner_type};

#[proc_macro_attribute]
pub fn nutype(attrs: proc_macro::TokenStream, type_definition: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_nutype(attrs.into(), type_definition.into()).into()
}

fn expand_nutype(attrs: TokenStream, type_definition: TokenStream) -> TokenStream {
    let TypeNameAndInnerType {
        type_name,
        inner_type,
    } = parse_type_name_and_inner_type(type_definition);
    let module_name = gen_module_name_for_type(&type_name);
    let meta = parse_attributes(attrs);

    // TODO: refactor!
    let error_type_import = if meta.validators.is_empty() {
        quote!()
    } else {
        let error_type_name = gen_error_type_name(&type_name);
        quote! (
            pub use #module_name::#error_type_name;
        )
    };

    let implementation = match inner_type {
        InnerType::String => {
            gen_string_implementation(&type_name, &meta)
        }
    };

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
