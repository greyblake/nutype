use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Generics;

use crate::common::{
    generate::generics::{SplitGenerics, add_bound_to_all_type_params},
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

    let generics_with_fromstr_bound = add_bound_to_all_type_params(
        generics,
        syn::parse_quote!(::core::str::FromStr<Err: ::core::fmt::Debug>),
    );

    let SplitGenerics {
        impl_generics: enum_generics,
        type_generics,
        where_clause,
    } = SplitGenerics::new(&generics_with_fromstr_bound);

    // Example for `struct Wrapper<T>(T) where T: Clone`:
    //
    // #[derive(Debug)]
    // pub enum WrapperParseError<T: FromStr<Err: Debug>> where T: Clone {
    //     Parse(<T as FromStr>::Err),
    //     Validate(WrapperError),
    // }
    let definition = if let Some(error_type_name) = maybe_error_type_name {
        quote! {
            #[derive(Debug)]
            pub enum #parse_error_type_name #enum_generics #where_clause {
                Parse(<#inner_type as ::core::str::FromStr>::Err),
                Validate(#error_type_name),
            }

            impl #enum_generics ::core::fmt::Display for #parse_error_type_name #type_generics #where_clause {
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
            #[derive(Debug)]
            pub enum #parse_error_type_name #enum_generics #where_clause {
                Parse(<#inner_type as ::core::str::FromStr>::Err),
            }

            impl #enum_generics ::core::fmt::Display for #parse_error_type_name #type_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #parse_error_type_name::Parse(err) => write!(f, "Failed to parse {}: {:?}", #type_name_str, err),
                    }
                }
            }
        }
    };

    cfg_if! {
        if #[cfg(any(ERROR_IN_CORE, feature = "std"))] {
            cfg_if! {
                if #[cfg(ERROR_IN_CORE)] {
                    let error = quote! { ::core::error::Error };
                } else {
                    let error = quote! { ::std::error::Error };
                }
            };
            let generics_with_fromstr_and_debug_bounds = add_bound_to_all_type_params(
                &generics_with_fromstr_bound,
                syn::parse_quote!(::core::fmt::Debug),
            );
            let SplitGenerics {
                impl_generics: error_impl_generics,
                type_generics: error_type_generics,
                where_clause: error_where_clause,
            } = SplitGenerics::new(&generics_with_fromstr_and_debug_bounds);
            let impl_error = quote! {
                impl #error_impl_generics #error for #parse_error_type_name #error_type_generics #error_where_clause {
                    fn source(&self) -> Option<&(dyn #error + 'static)> {
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
