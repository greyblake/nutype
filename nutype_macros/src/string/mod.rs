pub mod gen;
pub mod models;
pub mod parse;
pub mod validate;

use std::collections::HashSet;

use crate::common::models::{Attributes, DeriveTrait, GenerateParams, Newtype};

use darling::util::SpannedValue;
use models::{StringDeriveTrait, StringSanitizer, StringValidator};
use proc_macro2::TokenStream;

use self::{
    gen::gen_nutype_for_string, models::StringGuard, validate::validate_string_derive_traits,
};

pub struct StringNewtype;

impl Newtype for StringNewtype {
    type Sanitizer = StringSanitizer;
    type Validator = StringValidator;
    type TypedTrait = StringDeriveTrait;

    fn parse_attributes(attrs: TokenStream) -> Result<Attributes<StringGuard>, darling::Error> {
        parse::parse_attributes(attrs)
    }

    fn validate(
        guard: &StringGuard,
        derive_traits: Vec<SpannedValue<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, darling::Error> {
        validate_string_derive_traits(guard, derive_traits)
    }

    fn generate(params: GenerateParams<Self::TypedTrait, StringGuard>) -> TokenStream {
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
