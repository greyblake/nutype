use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Generate a name for the error which is used for FromStr trait implementation.
pub fn gen_parse_error_name(type_name: &Ident) -> Ident {
    let error_name = format!("{type_name}ParseError");
    Ident::new(&error_name, Span::call_site())
}

/// Generate an error which is used for FromStr trait implementation of non-string types (e.g.
/// floats or integers)
pub fn gen_def_parse_error(
    inner_type: &TokenStream,
    maybe_error_type_name: Option<&Ident>,
    parse_error_type_name: &Ident,
) -> TokenStream {
    let definition = if let Some(error_type_name) = maybe_error_type_name {
        quote! {
            #[derive(Debug)]
            pub enum #parse_error_type_name {
                Parse(<#inner_type as ::core::str::FromStr>::Err),
                Validate(#error_type_name),
            }

            impl ::core::fmt::Display for #parse_error_type_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    // TODO: display a meaningful error message, including:
                    // * type_name
                    // * display the inner error
                    // match self {
                    //     #parse_error_type_name::Parse(err) => write!(f, "parsing failed: {}", err),
                    //     #parse_error_type_name::Validate(err) => write!(f, "parsing failed: {}", err),
                    // }
                    //
                    write!(f, "Failed to parse")
                }
            }
        }
    } else {
        quote! {
            #[derive(Debug)]
            pub enum #parse_error_type_name {
                Parse(<#inner_type as ::core::str::FromStr>::Err),
            }

            impl ::core::fmt::Display for #parse_error_type_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    // TODO: display a meaningful error message, including:
                    // * type_name
                    // * display the inner error
                    // match self {
                    //     #parse_error_type_name::Parse(err) => write!(f, "parsing failed: {}", err),
                    // }

                    write!(f, "Failed to parse")
                }
            }
        }
    };

    let impl_std_error = quote! {
        impl ::std::error::Error for #parse_error_type_name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                None
            }
        }
    };

    quote! {
        #definition
        #impl_std_error
    }
}
