pub mod gen;
pub mod models;
pub mod parse;
pub mod validate;

use std::collections::HashSet;

use crate::common::{
    gen::GenerateNewtype,
    models::{Attributes, DeriveTrait, GenerateParams, Newtype, SpannedItem},
};

use models::{StringDeriveTrait, StringSanitizer, StringValidator};
use proc_macro2::TokenStream;

use self::{
    models::{StringGuard, StringInnerType},
    validate::validate_string_derive_traits,
};

pub struct StringNewtype;

impl Newtype for StringNewtype {
    type Sanitizer = StringSanitizer;
    type Validator = StringValidator;
    type TypedTrait = StringDeriveTrait;
    type InnerType = StringInnerType;

    fn parse_attributes(attrs: TokenStream) -> Result<Attributes<StringGuard>, syn::Error> {
        parse::parse_attributes(attrs)
    }

    fn validate(
        guard: &StringGuard,
        derive_traits: Vec<SpannedItem<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        validate_string_derive_traits(guard, derive_traits)
    }

    fn generate(
        params: GenerateParams<StringInnerType, Self::TypedTrait, StringGuard>,
    ) -> TokenStream {
        StringNewtype::gen_nutype(params)
    }
}
