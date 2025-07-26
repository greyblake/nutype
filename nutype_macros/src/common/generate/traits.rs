use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Generics;

use crate::common::{
    generate::{add_bound_to_all_type_params, strip_trait_bounds_on_generics},
    models::{ErrorTypePath, InnerType, TypeName},
};

use super::parse_error::{gen_def_parse_error, gen_parse_error_name};

/// Generated implementation of traits.
pub struct GeneratedTraits {
    /// Transparent traits that are simply derived.
    pub derive_transparent_traits: TokenStream,

    /// Implementation of traits.
    pub implement_traits: TokenStream,
}

/// Split traits into 2 groups for generation:
/// * Transparent traits can be simply derived, e.g. `derive(Debug)`.
/// * Irregular traits requires custom implementation to be generated.
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

pub fn gen_impl_trait_into(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl Into<InnerType>,
) -> TokenStream {
    let inner_type: InnerType = inner_type.into();

    // NOTE: We're getting blank implementation of
    //     Into<Inner> for Type
    // by implementing
    //     From<Type> for Inner
    quote! {
        impl #generics ::core::convert::From<#type_name #generics> for #inner_type {
            #[inline]
            fn from(value: #type_name #generics) -> Self {
                value.into_inner()
            }
        }
    }
}

pub fn gen_impl_trait_as_ref(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl ToTokens,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    quote! {
        impl #generics ::core::convert::AsRef<#inner_type> for #type_name #generics_without_bounds {  // impl<T: Ord> AsRef<Vec<T>> for Collection<T> {
            #[inline]                                                                                 //     #[inline]
            fn as_ref(&self) -> &#inner_type {                                                        //     fn as_ref(&self) -> &Vec<T> {
                &self.0                                                                               //         &self.0
            }                                                                                         //     }
        }                                                                                             // }
    }
}

pub fn gen_impl_trait_deref(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl ToTokens,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    quote! {
        impl #generics ::core::ops::Deref for #type_name #generics_without_bounds {  // impl<T: Ord> Deref for Collection<T> {
            type Target = #inner_type;                                               //     type Target = Vec<T>;
                                                                                     //
            #[inline]                                                                //     #[inline]
            fn deref(&self) -> &Self::Target {                                       //     fn deref(&self) -> &Self::Target {
                &self.0                                                              //         &self.0
            }                                                                        //     }
        }                                                                            // }
    }
}

pub fn gen_impl_trait_display(type_name: &TypeName, generics: &Generics) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    let generics_with_display_bound =
        add_bound_to_all_type_params(generics, syn::parse_quote!(::core::fmt::Display));
    quote! {
        impl #generics_with_display_bound ::core::fmt::Display for #type_name #generics_without_bounds {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                // A tiny wrapper function with trait boundary that improves error reporting.
                // It makes it clear for the end-user that the inner type has to implement Display
                // in order to derive display for the newtype.
                #[inline]
                fn display<T: ::core::fmt::Display>(f: &mut ::core::fmt::Formatter<'_>, val: &T) -> ::core::fmt::Result {
                    use ::core::fmt::Display;
                    val.fmt(f)
                }
                display(f, &self.0)
            }
        }
    }
}

pub fn gen_impl_trait_borrow(
    type_name: &TypeName,
    generics: &Generics,
    borrowed_type: impl ToTokens,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    quote! {
        impl #generics ::core::borrow::Borrow<#borrowed_type> for #type_name #generics_without_bounds {  // impl<T: Ord> Borrow<Vec<T>> for Collection<T> {
            #[inline]                                                                                    //     #[inline]
            fn borrow(&self) -> &#borrowed_type {                                                        //     fn borrow(&self) -> &Vec<T> {
                &self.0                                                                                  //         &self.0
            }                                                                                            //     }
        }                                                                                                // }
    }
}

pub fn gen_impl_trait_from(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl ToTokens,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    quote! {
        impl #generics ::core::convert::From<#inner_type> for #type_name #generics_without_bounds {  // impl<T: Ord> From<Vec<T>> for Collection<T> {
            #[inline]                                                                                //     #[inline]
            fn from(raw_value: #inner_type) -> Self {                                                //     fn from(raw_value: Vec<T>) -> Self {
                Self::new(raw_value)                                                                 //         Self::new(raw_value)
            }                                                                                        //     }
        }                                                                                            // }
    }
}

pub fn gen_impl_trait_try_from(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl ToTokens,
    maybe_error_type_name: Option<&ErrorTypePath>,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    match maybe_error_type_name {
        Some(error_type_name) => {
            // The case when there are validation
            //
            quote! {
                impl #generics ::core::convert::TryFrom<#inner_type> for #type_name #generics_without_bounds {
                    type Error = #error_type_name;

                    #[inline]
                    fn try_from(raw_value: #inner_type) -> ::core::result::Result<#type_name #generics_without_bounds, Self::Error> {
                        Self::try_new(raw_value)
                    }
                }
            }
        }
        None => {
            // The case when there are no validation
            //
            quote! {
                impl #generics ::core::convert::TryFrom<#inner_type> for #type_name #generics_without_bounds {
                    type Error = ::core::convert::Infallible;

                    #[inline]
                    fn try_from(raw_value: #inner_type) -> ::core::result::Result<#type_name #generics_without_bounds, Self::Error> {
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
    generics: &Generics,
    inner_type: impl Into<InnerType>,
    maybe_error_type_name: Option<&ErrorTypePath>,
) -> TokenStream {
    let inner_type: InnerType = inner_type.into();
    let parse_error_type_name = gen_parse_error_name(type_name);
    let def_parse_error = gen_def_parse_error(
        type_name,
        generics,
        inner_type.clone(),
        maybe_error_type_name,
        &parse_error_type_name,
    );

    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    let generics_with_fromstr_bound = add_bound_to_all_type_params(
        generics,
        syn::parse_quote!(::core::str::FromStr<Err: ::core::fmt::Debug>),
    );

    if let Some(_error_type_name) = maybe_error_type_name {
        // The case with validation
        quote! {
            #def_parse_error

            impl #generics_with_fromstr_bound ::core::str::FromStr for #type_name #generics_without_bounds {
                type Err = #parse_error_type_name #generics_without_bounds;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    let raw_value: #inner_type = raw_string.parse().map_err(#parse_error_type_name::Parse)?;
                    Self::try_new(raw_value).map_err(#parse_error_type_name::Validate)
                }
            }
        }
    } else {
        // The case without validation
        quote! {
            #def_parse_error

            impl #generics_with_fromstr_bound ::core::str::FromStr for #type_name #generics_without_bounds {
                type Err = #parse_error_type_name #generics_without_bounds;

                fn from_str(raw_string: &str) -> ::core::result::Result<Self, Self::Err> {
                    let value: #inner_type = raw_string.parse().map_err(#parse_error_type_name::Parse)?;
                    Ok(#type_name::new(value))
                }
            }
        }
    }
}

pub fn gen_impl_trait_serde_serialize(type_name: &TypeName, generics: &Generics) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    // Turn `<T>` into `<T: Serialize>`
    let all_generics_with_serialize_bound =
        add_bound_to_all_type_params(generics, syn::parse_quote!(::serde::Serialize));

    let type_name_str = type_name.to_string();
    quote! {
        impl #all_generics_with_serialize_bound ::serde::Serialize for #type_name #generics_without_bounds {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer
            {
                ::serde::ser::Serializer::serialize_newtype_struct(serializer, #type_name_str, &self.0)
            }
        }
    }
}

pub fn gen_impl_trait_serde_deserialize(
    type_name: &TypeName,
    type_generics: &Generics,
    inner_type: impl Into<InnerType>,
    maybe_error_type_name: Option<&ErrorTypePath>,
) -> TokenStream {
    let inner_type: InnerType = inner_type.into();
    let raw_value_to_result: TokenStream = if maybe_error_type_name.is_some() {
        let type_name_str = type_name.to_string();
        quote! {
            #type_name::try_new(raw_value).map_err(|validation_error| {
                // Add a hint about which type is causing the error,
                <DE::Error as serde::de::Error>::custom(core::format_args!("{validation_error} Expected valid {}", #type_name_str))
            })
        }
    } else {
        quote! {
            Ok(#type_name::new(raw_value))
        }
    };

    let expecting_str = format!("tuple struct {type_name}");
    let type_name_str = type_name.to_string();

    // type generics + 'de lifetime for Deserialize
    let all_generics = {
        let mut all_generics = type_generics.clone();
        all_generics.params.push(syn::parse_quote!('de));
        all_generics
    };
    let all_generics_without_bounds = strip_trait_bounds_on_generics(&all_generics);
    let type_generics_without_bounds = strip_trait_bounds_on_generics(type_generics);

    // Turn `<'de, T>` into `<'de, T: Deserialize<'de>>`
    let all_generics_with_deserialize_bound =
        add_bound_to_all_type_params(&all_generics, syn::parse_quote!(::serde::Deserialize<'de>));

    quote! {
        impl #all_generics_with_deserialize_bound ::serde::Deserialize<'de> for #type_name #type_generics_without_bounds {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
                struct __Visitor #all_generics {
                    marker: ::core::marker::PhantomData<#type_name #type_generics_without_bounds>,
                    lifetime: ::core::marker::PhantomData<&'de ()>,
                }

                impl #all_generics_with_deserialize_bound ::serde::de::Visitor<'de> for __Visitor #all_generics_without_bounds {
                    type Value = #type_name #type_generics_without_bounds;

                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        write!(formatter, #expecting_str)
                    }

                    fn visit_newtype_struct<DE>(self, deserializer: DE) -> ::core::result::Result<Self::Value, DE::Error>
                    where
                        DE: ::serde::Deserializer<'de>
                    {
                        let raw_value: #inner_type = match <#inner_type as ::serde::Deserialize>::deserialize(deserializer) {
                            Ok(val) => val,
                            Err(err) => return Err(err)
                        };
                        #raw_value_to_result
                    }
                }

                ::serde::de::Deserializer::deserialize_newtype_struct(
                    deserializer,
                    #type_name_str,
                    __Visitor {
                        marker: Default::default(),
                        lifetime: Default::default(),
                    }
                )
            }
        }
    }
}

pub fn gen_impl_trait_default(
    type_name: &TypeName,
    generics: &Generics,
    default_value: impl ToTokens,
    has_validation: bool,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);

    if has_validation {
        let tp = type_name.to_string();
        quote!(
            impl #generics ::core::default::Default for #type_name #generics_without_bounds {
                fn default() -> Self {
                    Self::try_new(#default_value)
                        .unwrap_or_else(|err| {
                            let tp = #tp;
                            panic!("\nDefault value for type `{tp}` is invalid.\nERROR: {err:?}\n");
                        })
                }
            }
        )
    } else {
        quote!(
            impl #generics ::core::default::Default for #type_name #generics_without_bounds {
                #[inline]
                fn default() -> Self {
                    Self::new(#default_value)
                }
            }
        )
    }
}
