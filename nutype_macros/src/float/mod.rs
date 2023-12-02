use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::{
    gen::GenerateNewtype,
    models::{Attributes, GenerateParams, Guard, Newtype, SpannedDeriveTrait},
};

use self::{
    models::{
        FloatDeriveTrait, FloatGuard, FloatInnerType, FloatSanitizer, FloatType, FloatValidator,
    },
    validate::validate_float_derive_traits,
};

pub mod gen;
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
    ) -> Result<Attributes<FloatGuard<T>, SpannedDeriveTrait>, syn::Error> {
        parse::parse_attributes::<T>(attrs)
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
