pub mod gen;
pub mod models;
pub mod parse;
pub mod validate;

use proc_macro2::TokenStream;
use std::collections::HashSet;

use self::models::{AnyDeriveTrait, AnyGuard, AnyInnerType, AnySanitizer, AnyValidator};
use crate::common::gen::GenerateNewtype;
use crate::common::models::TypeName;
use crate::{
    any::validate::validate_any_derive_traits,
    common::models::{Attributes, GenerateParams, Newtype, SpannedDeriveTrait},
};

pub struct AnyNewtype;

impl Newtype for AnyNewtype {
    type Sanitizer = AnySanitizer;
    type Validator = AnyValidator;
    type TypedTrait = AnyDeriveTrait;
    type InnerType = AnyInnerType;

    fn parse_attributes(
        attrs: TokenStream,
        type_name: &TypeName,
    ) -> Result<Attributes<AnyGuard, SpannedDeriveTrait>, syn::Error> {
        parse::parse_attributes(attrs, type_name)
    }

    fn validate(
        guard: &AnyGuard,
        derive_traits: Vec<SpannedDeriveTrait>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        validate_any_derive_traits(guard, derive_traits)
    }

    fn generate(
        params: GenerateParams<AnyInnerType, Self::TypedTrait, AnyGuard>,
    ) -> Result<TokenStream, syn::Error> {
        AnyNewtype::gen_nutype(params)
    }
}
