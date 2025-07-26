use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};
use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::{
    generate::GenerateNewtype,
    models::{Attributes, GenerateParams, Guard, Newtype, SpannedDeriveTrait, TypeName},
};

use self::{
    models::{
        FloatDeriveTrait, FloatGuard, FloatInnerType, FloatSanitizer, FloatType, FloatValidator,
    },
    validate::validate_float_derive_traits,
};

pub mod generate;
pub mod models;
pub mod parse;
pub mod validate;

pub struct FloatNewtype<T: FloatType>(PhantomData<T>);

impl<T> Newtype for FloatNewtype<T>
where
    T: FloatType + ToTokens + FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    type Sanitizer = FloatSanitizer<T>;
    type Validator = FloatValidator<T>;
    type TypedTrait = FloatDeriveTrait;
    type InnerType = FloatInnerType;

    fn parse_attributes(
        attrs: TokenStream,
        type_name: &TypeName,
    ) -> Result<Attributes<FloatGuard<T>, SpannedDeriveTrait>, syn::Error> {
        parse::parse_attributes::<T>(attrs, type_name)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedDeriveTrait>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        validate_float_derive_traits(derive_traits, guard)
    }

    fn generate(
        params: GenerateParams<
            FloatInnerType,
            Self::TypedTrait,
            Guard<Self::Sanitizer, Self::Validator>,
        >,
    ) -> Result<TokenStream, syn::Error> {
        FloatNewtype::gen_nutype(params)
    }
}
