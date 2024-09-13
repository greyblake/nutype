use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Generics;

use crate::common::{
    gen::{add_bound_to_all_type_params, strip_trait_bounds_on_generics},
    models::{ErrorTypePath, InnerType, ParseErrorTypeName, TypeName},
};

/// Generate a name for the error which is used for FromStr trait implementation.
pub fn gen_parse_error_name(type_name: &TypeName) -> ParseErrorTypeName {
    let ident = format_ident!("{type_name}ParseError");
    ParseErrorTypeName::new(ident)
}

/// Generate an error which is used for FromStr trait implementation of non-string types (e.g.
/// floats or integers)
pub fn gen_def_parse_error(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl Into<InnerType>,
    maybe_error_type_name: Option<&ErrorTypePath>,
    parse_error_type_name: &ParseErrorTypeName,
) -> TokenStream {
    let inner_type: InnerType = inner_type.into();
    let type_name_str = type_name.to_string();

    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    let generics_with_fromstr_bound = add_bound_to_all_type_params(
        &generics_without_bounds,
        syn::parse_quote!(::core::str::FromStr<Err: ::core::fmt::Debug>),
    );

    let definition = if let Some(error_type_name) = maybe_error_type_name {
        quote! {
            #[derive(Debug)]                                                                    // #[derive(Debug)]
            pub enum #parse_error_type_name #generics_with_fromstr_bound {                      // pub enum ParseErrorFoo<T: ::core::str::FromStr<Err: ::core::fmt::Debug>> {
                Parse(<#inner_type as ::core::str::FromStr>::Err),                              //     Parse(<Foo as ::core::str::FromStr>::Err),
                Validate(#error_type_name),                                                     //     Validate(ErrorFoo),
            }                                                                                   // }

            impl #generics_with_fromstr_bound ::core::fmt::Display for #parse_error_type_name #generics_without_bounds {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #parse_error_type_name::Parse(err) => write!(f, "Failed to parse {}: {:?}", #type_name_str, err),
                        #parse_error_type_name::Validate(err) => write!(f, "Failed to parse {}: {}", #type_name_str, err),
                    }

                }
            }
        }
    } else {
        quote! {
            #[derive(Debug)]                                                 // #[derive(Debug)
            pub enum #parse_error_type_name #generics_with_fromstr_bound {   // pub enum ParseErrorFoo<T: ::core::str::FromStr<Err: ::core::fmt::Debug>> {
                Parse(<#inner_type as ::core::str::FromStr>::Err),           //     Parse(<Foo as ::core::str::FromStr>::Err),
            }                                                                // }

            impl #generics_with_fromstr_bound ::core::fmt::Display for #parse_error_type_name #generics_without_bounds {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #parse_error_type_name::Parse(err) => write!(f, "Failed to parse {}: {:?}", #type_name_str, err),
                    }
                }
            }
        }
    };

    cfg_if! {
        if #[cfg(ERROR_IN_CORE)] {
            let generics_with_fromstr_and_debug_bounds = add_bound_to_all_type_params(
                &generics_with_fromstr_bound,
                syn::parse_quote!(::core::fmt::Debug),
            );
            let impl_error = quote! {
                impl #generics_with_fromstr_and_debug_bounds ::core::error::Error for #parse_error_type_name #generics_without_bounds {
                    fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
                        None
                    }
                }
            };
        } else if #[cfg(feature = "std")] {
            let generics_with_fromstr_and_debug_bounds = add_bound_to_all_type_params(
                &generics_with_fromstr_bound,
                syn::parse_quote!(::core::fmt::Debug),
            );
            let impl_error = quote! {
                impl #generics_with_fromstr_and_debug_bounds ::std::error::Error for #parse_error_type_name #generics_without_bounds {
                    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                        None
                    }
                }
            };
        } else {
            // NOTE: `::core::error::Error` is stable only for rust >= 1.81.0.
            let impl_error = quote! {};
        }
    };

    quote! {
        #definition
        #impl_error
    }
}
