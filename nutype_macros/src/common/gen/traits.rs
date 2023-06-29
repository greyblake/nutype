use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    common::models::{ErrorTypeName, InnerType, TypeName},
    utils::match_feature,
};

use super::parse_error::{gen_def_parse_error, gen_parse_error_name};

/// Generated implementation of traits.
pub struct GeneratedTraits {
    /// Transparent traits that are simply derived.
    pub derive_transparent_traits: TokenStream,

    /// Implementation of traits.
    pub implement_traits: TokenStream,
}

/// Split traits into 2 groups for generatation:
/// * Transparent traits can be simply derived, e.g. `derive(Debug)`.
/// * Irregular traits requires implementation to be generated.
pub enum GeneratableTrait<TransparentTrait, IrregularTrait> {
    Transparent(TransparentTrait),
    Irregular(IrregularTrait),
}

pub struct GeneratableTraits<TransparentTrait, IrregularTrait> {
    pub transparent_traits: Vec<TransparentTrait>,
    pub irregular_traits: Vec<IrregularTrait>,
}

pub fn split_into_generatable_traits<InputTrait, TransparentTrait, IrregularTrait>(
    input_traits: HashSet<InputTrait>,
) -> GeneratableTraits<TransparentTrait, IrregularTrait>
where
    GeneratableTrait<TransparentTrait, IrregularTrait>: From<InputTrait>,
{
    let mut transparent_traits: Vec<TransparentTrait> = Vec::with_capacity(24);
    let mut irregular_traits: Vec<IrregularTrait> = Vec::with_capacity(24);

    for input_trait in input_traits {
        match GeneratableTrait::from(input_trait) {
            GeneratableTrait::Transparent(st) => transparent_traits.push(st),
            GeneratableTrait::Irregular(it) => irregular_traits.push(it),
        };
    }

    GeneratableTraits {
        transparent_traits,
        irregular_traits,
    }
}

pub fn gen_impl_trait_into(type_name: &TypeName, inner_type: impl Into<InnerType>) -> TokenStream {
    let inner_type: InnerType = inner_type.into();

    // NOTE: We're getting blank implementation of
    //     Into<Inner> for Type
    // by implementing
    //     From<Type> for Inner
    quote! {
        impl ::core::convert::From<#type_name> for #inner_type {
            fn from(value: #type_name) -> Self {
                value.into_inner()
            }
        }
    }
}

pub fn gen_impl_trait_as_ref(type_name: &TypeName, inner_type: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::convert::AsRef<#inner_type> for #type_name {
            fn as_ref(&self) -> &#inner_type {
                &self.0
            }
        }
    }
}

pub fn gen_impl_trait_deref(type_name: &TypeName, inner_type: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::ops::Deref for #type_name {
            type Target = #inner_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}

pub fn gen_impl_trait_dislpay(type_name: &TypeName) -> TokenStream {
    quote! {
        impl ::core::fmt::Display for #type_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                use ::core::fmt::Display;
                self.0.fmt(f)
            }
        }
    }
}

pub fn gen_impl_trait_borrow(type_name: &TypeName, borrowed_type: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::borrow::Borrow<#borrowed_type> for #type_name {
            fn borrow(&self) -> &#borrowed_type {
                &self.0
            }
        }
    }
}

pub fn gen_impl_trait_from(type_name: &TypeName, inner_type: impl ToTokens) -> TokenStream {
    quote! {
        impl ::core::convert::From<#inner_type> for #type_name {
            fn from(raw_value: #inner_type) -> Self {
                Self::new(raw_value)
            }
        }
    }
}

pub fn gen_impl_trait_try_from(
    type_name: &TypeName,
    inner_type: impl ToTokens,
    maybe_error_type_name: Option<&ErrorTypeName>,
) -> TokenStream {
    match maybe_error_type_name {
        Some(error_type_name) => {
            // The case when there are validation
            //
            quote! {
                impl ::core::convert::TryFrom<#inner_type> for #type_name {
                    type Error = #error_type_name;

                    fn try_from(raw_value: #inner_type) -> Result<#type_name, Self::Error> {
                        Self::new(raw_value)
                    }
                }
            }
        }
        None => {
            // The case when there are no validation
            //
            quote! {
                impl ::core::convert::TryFrom<#inner_type> for #type_name {
                    type Error = ::core::convert::Infallible;

                    fn try_from(raw_value: #inner_type) -> Result<#type_name, Self::Error> {
                        Ok(Self::new(raw_value))
                    }
                }
            }
        }
    }
}

/// Generate implementation of FromStr trait for non-string types (e.g. integers or floats).
pub fn gen_impl_trait_from_str(
    type_name: &TypeName,
    inner_type: impl Into<InnerType>,
    maybe_error_type_name: Option<&ErrorTypeName>,
) -> TokenStream {
    let inner_type: InnerType = inner_type.into();
    let parse_error_type_name = gen_parse_error_name(type_name);
    let def_parse_error = gen_def_parse_error(
        inner_type,
        type_name,
        maybe_error_type_name,
        &parse_error_type_name,
    );

    if let Some(_error_type_name) = maybe_error_type_name {
        // The case with validation
        quote! {
            #def_parse_error

            impl ::core::str::FromStr for #type_name {
                type Err = #parse_error_type_name;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    let raw_value: #inner_type = raw_string.parse().map_err(#parse_error_type_name::Parse)?;
                    Self::new(raw_value).map_err(#parse_error_type_name::Validate)
                }
            }
        }
    } else {
        // The case without validation
        quote! {
            #def_parse_error

            impl ::core::str::FromStr for #type_name {
                type Err = #parse_error_type_name;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    let value: #inner_type = raw_string.parse().map_err(#parse_error_type_name::Parse)?;
                    Ok(#type_name::new(value))
                }
            }
        }
    }
}

pub fn gen_impl_trait_serde_serialize(type_name: &TypeName) -> TokenStream {
    let type_name_str = type_name.to_string();
    quote! {
        impl ::serde::Serialize for #type_name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer
            {
                serializer.serialize_newtype_struct(#type_name_str, &self.0)
            }
        }
    }
}

pub fn gen_impl_trait_serde_deserialize(
    type_name: &TypeName,
    inner_type: impl Into<InnerType>,
    maybe_error_type_name: Option<&ErrorTypeName>,
) -> TokenStream {
    let inner_type: InnerType = inner_type.into();
    let raw_value_to_result: TokenStream = if maybe_error_type_name.is_some() {
        quote! {
            #type_name::new(raw_value).map_err(<D::Error as serde::de::Error>::custom)
        }
    } else {
        quote! {
            Ok(#type_name::new(raw_value))
        }
    };

    quote! {
        impl<'de> ::serde::Deserialize<'de> for #type_name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw_value = #inner_type::deserialize(deserializer)?;
                #raw_value_to_result
            }
        }
    }
}

pub fn gen_impl_trait_default(
    type_name: &TypeName,
    default_value: impl ToTokens,
    has_validation: bool,
) -> TokenStream {
    let default_implementation = if has_validation {
        let tp = type_name.to_string();
        quote!(
            Self::new(#default_value)
                .unwrap_or_else(|err| {
                    let tp = #tp;
                    panic!("\nDefault value for type {tp} is invalid.\nERROR: {err}\n")
                })
        )
    } else {
        quote!(
            Self::new(#default_value)
        )
    };

    // Unfortunately it's not possible to guarantee at the compile time that the default value will always
    // satisfy the validation rules.
    // For this purpose we generate a unit test to verify this at run time.
    let unit_test = if has_validation {
        match_feature!("nutype_test",
            on  => quote!(),
            off => quote!(
                #[test]
                fn should_have_valid_default_value() {
                    let default_inner_value = #type_name::default().into_inner();
                    #type_name::new(default_inner_value).expect("Default value must be valid");
                }
            )
        )
    } else {
        quote!()
    };

    quote!(
        impl ::core::default::Default for #type_name {
            fn default() -> Self {
                #default_implementation
            }
        }
        #unit_test
    )
}
