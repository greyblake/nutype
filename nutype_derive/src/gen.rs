use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::models::{NewtypeStringMeta, StringSanitizer, StringValidator};

pub fn gen_string_implementation(
    type_name: &Ident,
    meta: &NewtypeStringMeta,
) -> TokenStream2 {
    if meta.validators.is_empty() {
        gen_string_from_implementation(type_name, &meta.sanitizers)
    } else {
        gen_string_try_from_implementation(
            type_name,
            &meta.sanitizers,
            &meta.validators,
        )
    }
}

fn gen_string_from_implementation(
    type_name: &Ident,
    sanitizers: &[StringSanitizer],
) -> TokenStream2 {
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
) -> TokenStream2 {
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

pub fn gen_string_sanitize_fn(sanitizers: &[StringSanitizer]) -> TokenStream2 {
    let transformations: TokenStream2 = sanitizers
        .into_iter()
        .map(|san| match san {
            StringSanitizer::Trim => {
                quote!(
                    let value: String = value.trim().to_string();
                )
            }
            StringSanitizer::Lowecase => {
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

pub fn gen_string_validate_fn(type_name: &Ident, validators: &[StringValidator]) -> TokenStream2 {
    let error_name = gen_error_type_name(type_name);

    let validations: TokenStream2 = validators
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

pub fn gen_validation_error_type(
    type_name: &Ident,
    validators: &[StringValidator],
) -> TokenStream2 {
    let error_name = gen_error_type_name(type_name);

    let error_variants: TokenStream2 = validators
        .into_iter()
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
