pub mod models;

use proc_macro2::TokenStream;
use std::collections::HashSet;

use self::models::{AnyDeriveTrait, AnyGuard, AnyInnerType, AnySanitizer, AnyValidator};
use crate::common::models::{Attributes, GenerateParams, Newtype, SpannedDeriveTrait};

pub struct AnyNewtype;

impl Newtype for AnyNewtype {
    type Sanitizer = AnySanitizer;
    type Validator = AnyValidator;
    type TypedTrait = AnyDeriveTrait;
    type InnerType = AnyInnerType;

    fn parse_attributes(
        attrs: TokenStream,
    ) -> Result<Attributes<AnyGuard, SpannedDeriveTrait>, syn::Error> {
        //parse::parse_attributes(attrs)
        todo!()
    }

    fn validate(
        guard: &AnyGuard,
        derive_traits: Vec<SpannedDeriveTrait>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        //validate_string_derive_traits(guard, derive_traits)
        todo!()
    }

    fn generate(params: GenerateParams<AnyInnerType, Self::TypedTrait, AnyGuard>) -> TokenStream {
        // StringNewtype::gen_nutype(params)
        todo!()
    }
}
