use std::{collections::HashSet, fmt::Debug, marker::PhantomData, str::FromStr};

use darling::util::SpannedValue;
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::models::{Attributes, DeriveTrait, GenerateParams, Guard, Newtype};

use self::{
    gen::gen_nutype_for_float,
    models::{FloatDeriveTrait, FloatGuard, FloatSanitizer, FloatType, FloatValidator},
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
    <T as FromStr>::Err: Debug,
{
    type Sanitizer = FloatSanitizer<T>;
    type Validator = FloatValidator<T>;
    type TypedTrait = FloatDeriveTrait;

    fn parse_attributes(attrs: TokenStream) -> Result<Attributes<FloatGuard<T>>, darling::Error> {
        parse::parse_attributes::<T>(attrs)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedValue<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, darling::Error> {
        validate_float_derive_traits(derive_traits, guard)
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

        let inner_type = T::float_inner_type();

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
