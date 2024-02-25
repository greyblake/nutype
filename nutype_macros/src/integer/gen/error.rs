use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::super::models::IntegerValidator;
use crate::common::{
    gen::error::{gen_error_type_name, gen_impl_error_trait},
    models::{ErrorTypeName, TypeName},
};

pub fn gen_validation_error_type<T: ToTokens>(
    type_name: &TypeName,
    validators: &[IntegerValidator<T>],
) -> TokenStream {
    let error_type_name = gen_error_type_name(type_name);
    let definition = gen_definition(&error_type_name, validators);
    let impl_display_trait = gen_impl_display_trait(type_name, &error_type_name, validators);
    let impl_error_trait = gen_impl_error_trait(&error_type_name);

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #definition

        #impl_display_trait
        #impl_error_trait
    }
}

fn gen_definition<T>(
    error_type_name: &ErrorTypeName,
    validators: &[IntegerValidator<T>],
) -> TokenStream {
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator {
            IntegerValidator::Greater(_) => {
                quote!(GreaterViolated,)
            }
            IntegerValidator::GreaterOrEqual(_) => {
                quote!(GreaterOrEqualViolated,)
            }
            IntegerValidator::Less(_) => {
                quote!(LessViolated,)
            }
            IntegerValidator::LessOrEqual(_) => {
                quote!(LessOrEqualViolated,)
            }
            IntegerValidator::Predicate(_) => {
                quote!(PredicateViolated,)
            }
        })
        .collect();

    quote! {
        #[allow(clippy::enum_variant_names)]
        pub enum #error_type_name {
            #error_variants
        }
    }
}

fn gen_impl_display_trait<T: ToTokens>(
    type_name: &TypeName,
    error_type_name: &ErrorTypeName,
    validators: &[IntegerValidator<T>],
) -> TokenStream {
    let match_arms = validators.iter().map(|validator| match validator {
        IntegerValidator::Greater(val) => quote! {
             #error_type_name::GreaterViolated => write!(f, "{} is too small. The value must be greater than {:#?}.", stringify!(#type_name), #val)
        },
        IntegerValidator::GreaterOrEqual(val) => quote! {
             #error_type_name::GreaterOrEqualViolated => write!(f, "{} is too small. The value must be greater or equal to {:#?}.", stringify!(#type_name), #val)
        },
        IntegerValidator::Less(val) => quote! {
             #error_type_name::LessViolated=> write!(f, "{} is too big. The value must be less than {:#?}.", stringify!(#type_name), #val)
        },
        IntegerValidator::LessOrEqual(val) => quote! {
             #error_type_name::LessOrEqualViolated=> write!(f, "{} is too big. The value must be less or equal to {:#?}.", stringify!(#type_name), #val)
        },
        IntegerValidator::Predicate(_) => quote! {
             #error_type_name::PredicateViolated => write!(f, "{} failed the predicate test.", stringify!(#type_name))
        },
    });

    quote! {
        impl ::core::fmt::Display for #error_type_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    }
}
