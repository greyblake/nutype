use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use super::models::{NewtypeNumberMeta, NumberSanitizer, NumberValidator};
use crate::models::NumberType;

pub fn gen_nutype_for_number<T>(
    number_type: NumberType,
    type_name: &Ident,
    meta: NewtypeNumberMeta<T>,
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let module_name = gen_module_name_for_type(type_name);
    let implementation = gen_implementation(type_name, &meta);

    // TODO: refactor: inject InnerType, that implements ToString
    let tp: TokenStream =
        syn::parse_str(std::any::type_name::<T>()).expect("Expected to parse a type");

    let error_type_import = match meta {
        NewtypeNumberMeta::From { .. } => quote!(),
        NewtypeNumberMeta::TryFrom { .. } => {
            let error_type_name = gen_error_type_name(type_name);
            quote! (
                pub use #module_name::#error_type_name;
            )
        }
    };
    let derive = gen_derive(number_type);

    quote!(
        mod #module_name {
            // TODO: respect visiblity!
            #derive
            pub struct #type_name(#tp);

            #implementation
        }
        pub use #module_name::#type_name;
        #error_type_import
    )
}

pub fn gen_implementation<T>(type_name: &Ident, meta: &NewtypeNumberMeta<T>) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    match meta {
        NewtypeNumberMeta::From { sanitizers } => gen_from_implementation(type_name, sanitizers),
        NewtypeNumberMeta::TryFrom {
            sanitizers,
            validators,
        } => gen_try_from_implementation(type_name, sanitizers, validators),
    }
}

fn gen_from_implementation<T>(type_name: &Ident, sanitizers: &[NumberSanitizer<T>]) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let tp: TokenStream =
        syn::parse_str(std::any::type_name::<T>()).expect("Expected to parse a type");
    let sanitize = gen_sanitize_fn(sanitizers);

    quote!(
        #sanitize

        impl ::core::convert::From<#tp> for #type_name {
            fn from(raw_value: #tp) -> #type_name {
                #type_name(sanitize(raw_value))
            }
        }
    )
}

fn gen_try_from_implementation<T>(
    type_name: &Ident,
    sanitizers: &[NumberSanitizer<T>],
    validators: &[NumberValidator<T>],
) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let tp: TokenStream =
        syn::parse_str(std::any::type_name::<T>()).expect("Expected to parse a type");
    let sanitize = gen_sanitize_fn(sanitizers);
    let validation_error = gen_validation_error_type(type_name, validators);
    let error_type_name = gen_error_type_name(type_name);
    let validate = gen_validate_fn(type_name, validators);

    quote!(
        #sanitize
        #validation_error
        #validate

        impl ::core::convert::TryFrom<#tp> for #type_name {
            type Error = #error_type_name;

            fn try_from(raw_value: #tp) -> Result<#type_name, Self::Error> {
                let sanitized_value = sanitize(raw_value);
                validate(sanitized_value)?;
                Ok(#type_name(sanitized_value))
            }
        }
    )
}

// TODO: DRY
fn gen_module_name_for_type(type_name: &Ident) -> Ident {
    let module_name = format!("__nutype_module_for_{type_name}");
    Ident::new(&module_name, Span::call_site())
}

fn gen_sanitize_fn<T>(sanitizers: &[NumberSanitizer<T>]) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let tp: TokenStream =
        syn::parse_str(std::any::type_name::<T>()).expect("Expected to parse a type");
    let transformations: TokenStream = sanitizers
        .iter()
        .map(|san| match san {
            NumberSanitizer::Clamp { min, max } => {
                quote!(
                    value = value.clamp(#min, #max);
                )
            }
        })
        .collect();

    quote!(
        fn sanitize(mut value: #tp) -> #tp {
            #transformations
            value
        }
    )
}

fn gen_error_type_name(type_name: &Ident) -> Ident {
    let error_name_str = format!("{type_name}Error");
    Ident::new(&error_name_str, Span::call_site())
}

fn gen_validate_fn<T>(type_name: &Ident, validators: &[NumberValidator<T>]) -> TokenStream
where
    T: ToTokens + PartialOrd,
{
    let error_name = gen_error_type_name(type_name);
    let tp: TokenStream =
        syn::parse_str(std::any::type_name::<T>()).expect("Expected to parse a type");

    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            NumberValidator::Max(max) => {
                quote!(
                    if val > #max {
                        return Err(#error_name::TooBig);
                    }
                )
            }
            NumberValidator::Min(min) => {
                quote!(
                    if val < #min {
                        return Err(#error_name::TooSmall);
                    }
                )
            }
        })
        .collect();

    quote!(
        fn validate(val: #tp) -> Result<(), #error_name> {
            #validations
            Ok(())
        }
    )
}

fn gen_validation_error_type<T>(
    type_name: &Ident,
    validators: &[NumberValidator<T>],
) -> TokenStream {
    let error_name = gen_error_type_name(type_name);

    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            NumberValidator::Min(_) => {
                quote!(TooSmall,)
            }
            NumberValidator::Max(_) => {
                quote!(TooBig,)
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

fn gen_derive(number_type: NumberType) -> TokenStream {
    use NumberType::*;

    match number_type {
        U8 | U16 | U32 | U64 | U128 | I8 | I16 | I32 | I64 | I128 | Usize | Isize => {
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            }
        }
    }
}
