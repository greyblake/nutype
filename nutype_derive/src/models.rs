use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::{TokenStream as TokenStream2, Ident};

pub enum StringSanitizer {
    Trim,
    Lowecase,
    Uppercase,
}

pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    // Present (aka NotEmpty)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName(String);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerType {
    String
}

impl ToTokens for InnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream2) {

        match self {
            InnerType::String => {
                quote!(String).to_tokens(token_stream);
            },
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNameAndInnerType {
    pub type_name: Ident,
    pub inner_type: InnerType,
}

pub struct NewtypeDefinition {
    pub name: String,
    pub meta: NewtypeMeta,
}

pub enum NewtypeMeta {
    String(NewtypeStringMeta)
}

pub struct NewtypeStringMeta {
    pub sanitizers: Vec<StringSanitizer>,
    pub validators: Vec<StringValidator>,
}

