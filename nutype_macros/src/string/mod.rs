pub mod generate;
pub mod models;
pub mod parse;
pub mod validate;

use crate::common::{
    generate::GenerateNewtype,
    models::{
        Attributes, CfgAttrEntry, GenerateParams, Newtype, SpannedDeriveTrait, TypeName,
        ValidatedDerives,
    },
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

    fn parse_attributes(
        attrs: TokenStream,
        type_name: &TypeName,
    ) -> Result<Attributes<StringGuard, SpannedDeriveTrait>, syn::Error> {
        parse::parse_attributes(attrs, type_name)
    }

    fn validate(
        guard: &StringGuard,
        derive_traits: Vec<SpannedDeriveTrait>,
        cfg_attr_entries: &[CfgAttrEntry],
        maybe_default_value: &Option<syn::Expr>,
        type_name: &TypeName,
    ) -> Result<ValidatedDerives<Self::TypedTrait>, syn::Error> {
        validate_string_derive_traits(
            guard,
            derive_traits,
            cfg_attr_entries,
            maybe_default_value,
            type_name,
        )
    }

    fn generate(
        params: GenerateParams<StringInnerType, Self::TypedTrait, StringGuard>,
    ) -> Result<TokenStream, syn::Error> {
        StringNewtype::gen_nutype(params)
    }
}
