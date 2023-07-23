use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::models::{Attributes, DeriveTrait, GenerateParams, Guard, Newtype, SpannedItem};

use self::{
    gen::gen_nutype_for_float,
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

    fn parse_attributes(attrs: TokenStream) -> Result<Attributes<FloatGuard<T>>, syn::Error> {
        parse::parse_attributes::<T>(attrs)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedItem<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        validate_float_derive_traits(derive_traits, guard)
    }

    fn generate(
        params: GenerateParams<
            FloatInnerType,
            Self::TypedTrait,
            Guard<Self::Sanitizer, Self::Validator>,
        >,
    ) -> TokenStream {
        let GenerateParams {
            doc_attrs,
            traits,
            vis,
            type_name,
            guard,
            new_unchecked,
            maybe_default_value,
            inner_type,
        } = params;

        gen_nutype_for_float(
            doc_attrs,
            vis,
            inner_type,
            &type_name,
            guard,
            traits,
            new_unchecked,
            maybe_default_value,
        )
    }
}
