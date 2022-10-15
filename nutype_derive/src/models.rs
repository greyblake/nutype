use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};

#[derive(Debug)]
pub enum StringSanitizer {
    Trim,
    Lowecase,
    Uppercase,
}

#[derive(Debug)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    Present,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerType {
    String,
}

impl ToTokens for InnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream2) {
        match self {
            InnerType::String => {
                quote!(String).to_tokens(token_stream);
            }
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNameAndInnerType {
    pub type_name: Ident,
    pub inner_type: InnerType,
}

// #[derive(Debug)]
// pub struct NewtypeDefinition {
//     pub name: String,
//     pub meta: NewtypeMeta,
// }
//
// #[derive(Debug)]
// pub enum NewtypeMeta {
//     String(NewtypeStringMeta)
// }

#[derive(Debug)]
pub struct NewtypeStringMeta {
    pub sanitizers: Vec<StringSanitizer>,
    pub validators: Vec<StringValidator>,
}
