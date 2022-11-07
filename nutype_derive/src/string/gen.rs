use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::models::{StringSanitizer, StringValidator};

use super::models::NewtypeStringMeta;

pub fn gen_nutype_for_string(type_name: &Ident, meta: NewtypeStringMeta) -> TokenStream {
    let module_name = gen_module_name_for_type(type_name);
    let implementation = gen_string_implementation(type_name, &meta);

    let error_type_import = match meta {
        NewtypeStringMeta::From { .. } => quote!(),
        NewtypeStringMeta::TryFrom { .. } => {
            let error_type_name = gen_error_type_name(type_name);
            quote! (
                pub use #module_name::#error_type_name;
            )
        }
    };

    quote!(
        mod #module_name {
            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
            // TODO: respect visiblity!
            pub struct #type_name(String);

            #implementation
        }
        pub use #module_name::#type_name;
        #error_type_import
    )
}

pub fn gen_string_implementation(type_name: &Ident, meta: &NewtypeStringMeta) -> TokenStream {
    match meta {
        NewtypeStringMeta::From { sanitizers } => {
            gen_string_from_implementation(type_name, sanitizers)
        }
        NewtypeStringMeta::TryFrom {
            sanitizers,
            validators,
        } => gen_string_try_from_implementation(type_name, sanitizers, validators),
    }
}

fn gen_string_from_implementation(
    type_name: &Ident,
    sanitizers: &[StringSanitizer],
) -> TokenStream {
    let sanitize = gen_string_sanitize_fn(sanitizers);

    quote!(
        #sanitize

        impl ::core::convert::From<String> for #type_name {
            fn from(raw_value: String) -> #type_name {
                #type_name(sanitize(raw_value))
            }
        }

        impl ::core::convert::From<&str> for #type_name {

            fn from(raw_value: &str) -> #type_name {
                let raw_value = raw_value.to_string();
                #type_name(sanitize(raw_value))
            }
        }
    )
}

fn gen_string_try_from_implementation(
    type_name: &Ident,
    sanitizers: &[StringSanitizer],
    validators: &[StringValidator],
) -> TokenStream {
    let sanitize = gen_string_sanitize_fn(sanitizers);
    let validation_error = gen_validation_error_type(type_name, validators);
    let error_type_name = gen_error_type_name(type_name);
    let validate = gen_string_validate_fn(type_name, validators);

    quote!(
        #sanitize
        #validation_error
        #validate

        impl ::core::convert::TryFrom<String> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: String) -> Result<#type_name, Self::Error> {
                let sanitized_value = sanitize(raw_value);
                validate(&sanitized_value)?;
                Ok(#type_name(sanitized_value))
            }
        }

        impl ::core::convert::TryFrom<&str> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: &str) -> Result<#type_name, Self::Error> {
                let raw_value = raw_value.to_string();
                let sanitized_value = sanitize(raw_value);
                validate(&sanitized_value)?;
                Ok(#type_name(sanitized_value))
            }
        }
    )
}

pub fn gen_module_name_for_type(type_name: &Ident) -> Ident {
    let module_name = format!("__nutype_module_for_{type_name}");
    Ident::new(&module_name, Span::call_site())
}

pub fn gen_string_sanitize_fn(sanitizers: &[StringSanitizer]) -> TokenStream {
    let transformations: TokenStream = sanitizers
        .iter()
        .map(|san| match san {
            StringSanitizer::Trim => {
                quote!(
                    let value: String = value.trim().to_string();
                )
            }
            StringSanitizer::Lowercase => {
                quote!(
                    let value: String = value.to_lowercase();
                )
            }
            StringSanitizer::Uppercase => {
                quote!(
                    let value: String = value.to_uppercase();
                )
            }
        })
        .collect();

    quote!(
        fn sanitize(value: String) -> String {
            #transformations
            value
        }
    )
}

pub fn gen_error_type_name(type_name: &Ident) -> Ident {
    let error_name_str = format!("{type_name}Error");
    Ident::new(&error_name_str, Span::call_site())
}

pub fn gen_string_validate_fn(type_name: &Ident, validators: &[StringValidator]) -> TokenStream {
    let error_name = gen_error_type_name(type_name);

    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::MaxLen(max_len) => {
                quote!(
                    if val.len() > #max_len {
                        return Err(#error_name::TooLong);
                    }
                )
            }
            StringValidator::MinLen(min_len) => {
                quote!(
                    if val.len() < #min_len {
                        return Err(#error_name::TooShort);
                    }
                )
            }
            StringValidator::Present => {
                quote!(
                    if val.is_empty() {
                        return Err(#error_name::Missing);
                    }
                )
            }
        })
        .collect();

    quote!(
        fn validate(val: &str) -> Result<(), #error_name> {
            #validations
            Ok(())
        }
    )
}

pub fn gen_validation_error_type(type_name: &Ident, validators: &[StringValidator]) -> TokenStream {
    let error_name = gen_error_type_name(type_name);

    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            StringValidator::MaxLen(_len) => {
                quote!(TooLong,)
            }
            StringValidator::MinLen(_len) => {
                quote!(TooShort,)
            }
            StringValidator::Present => {
                quote!(Missing,)
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
