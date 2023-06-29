pub mod gen;
pub mod models;
pub mod parse;
pub mod validate;

use std::collections::HashSet;

use crate::{
    common::models::{Attributes, DeriveTrait, Guard, SpannedItem},
    GenerateParams, Newtype,
};

use models::{StringDeriveTrait, StringSanitizer, StringValidator};
use proc_macro2::TokenStream;

use self::{gen::gen_nutype_for_string, validate::validate_string_derive_traits};

pub struct StringNewtype;

impl Newtype for StringNewtype {
    type Sanitizer = StringSanitizer;
    type Validator = StringValidator;
    type TypedTrait = StringDeriveTrait;

    fn parse_attributes(
        attrs: TokenStream,
    ) -> Result<Attributes<Guard<Self::Sanitizer, Self::Validator>>, syn::Error> {
        parse::parse_attributes(attrs)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedItem<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        validate_string_derive_traits(guard, derive_traits)
    }

    fn generate(
        params: GenerateParams<Self::TypedTrait, Guard<Self::Sanitizer, Self::Validator>>,
    ) -> TokenStream {
        let GenerateParams {
            doc_attrs,
            traits,
            vis,
            type_name,
            guard,
            new_unchecked,
            maybe_default_value,
        } = params;
        gen_nutype_for_string(
            doc_attrs,
            traits,
            vis,
            &type_name,
            guard,
            new_unchecked,
            maybe_default_value,
        )
    }
}
