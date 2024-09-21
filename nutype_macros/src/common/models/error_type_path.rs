use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, parse::{Parse, ParseStream}};
use quote::ToTokens;

// Represents a path to an error type.
// Could be a single Ident (e.g. `NameError`, but could also be a path (e.g. `std::io::Error`).
#[derive(Debug, Clone)]
pub struct ErrorTypePath(syn::Path);

impl ErrorTypePath {
    pub fn new(name: impl Into<syn::Path>) -> Self {
        Self(name.into())
    }

    pub fn span(&self) -> Span {
        self.0.span()
    }
}

impl Parse for ErrorTypePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<syn::Path>()?;
        Ok(Self::new(path))
    }
}

impl core::fmt::Display for ErrorTypePath {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let token_stream: TokenStream = self.0.clone().to_token_stream();
        write!(f, "{}", token_stream)
    }
}

impl ::quote::ToTokens for ErrorTypePath {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        self.0.to_tokens(token_stream)
    }
}
