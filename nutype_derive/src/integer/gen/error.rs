use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::super::models::IntegerValidator;

pub fn gen_error_type_name(type_name: &Ident) -> Ident {
    let error_name_str = format!("{type_name}Error");
    Ident::new(&error_name_str, Span::call_site())
}

pub fn gen_validation_error_type<T>(
    type_name: &Ident,
    validators: &[IntegerValidator<T>],
) -> TokenStream {
    let error_name = gen_error_type_name(type_name);

    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            IntegerValidator::Min(_) => {
                quote!(TooSmall,)
            }
            IntegerValidator::Max(_) => {
                quote!(TooBig,)
            }
            IntegerValidator::With(_) => {
                quote!(Invalid,)
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum #error_name {
            #error_variants
        }
    }
}
