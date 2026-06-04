use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::{
    generate::GenerateNewtype,
    models::{
        Attributes, CfgAttrEntry, GenerateParams, Guard, Newtype, SpannedDeriveTrait, TypeName,
        ValidatedDerives,
    },
};

use self::{
    models::{
        DecimalDeriveTrait, DecimalGuard, DecimalInnerType, DecimalSanitizer, DecimalType,
        DecimalValidator,
    },
    validate::validate_decimal_derive_traits,
};

pub mod generate;
pub mod models;
pub mod parse;
pub mod validate;

pub struct DecimalNewtype<T: DecimalType>(PhantomData<T>);

impl<T> Newtype for DecimalNewtype<T>
where
    T: DecimalType + ToTokens + FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    type Sanitizer = DecimalSanitizer<T>;
    type Validator = DecimalValidator<T>;
    type TypedTrait = DecimalDeriveTrait;
    type InnerType = DecimalInnerType;

    fn parse_attributes(
        attrs: TokenStream,
        type_name: &TypeName,
    ) -> Result<Attributes<DecimalGuard<T>, SpannedDeriveTrait>, syn::Error> {
        parse::parse_attributes::<T>(attrs, type_name)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedDeriveTrait>,
        cfg_attr_entries: &[CfgAttrEntry],
        maybe_default_value: &Option<syn::Expr>,
        type_name: &TypeName,
    ) -> Result<ValidatedDerives<Self::TypedTrait>, syn::Error> {
        let has_validation = guard.has_validation();
        validate_decimal_derive_traits(
            derive_traits,
            has_validation,
            cfg_attr_entries,
            maybe_default_value,
            type_name,
        )
    }

    fn generate(
        params: GenerateParams<
            DecimalInnerType,
            Self::TypedTrait,
            Guard<Self::Sanitizer, Self::Validator>,
        >,
    ) -> Result<TokenStream, syn::Error> {
        DecimalNewtype::gen_nutype(params)
    }
}
