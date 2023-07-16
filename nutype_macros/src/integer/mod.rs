use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::{
    common::models::{Attributes, DeriveTrait, GenerateParams, Guard, Newtype, SpannedItem},
    integer::gen::gen_nutype_for_integer,
};

use self::{
    models::{IntegerDeriveTrait, IntegerGuard, IntegerSanitizer, IntegerType, IntegerValidator},
    validate::validate_integer_derive_traits,
};

pub mod gen;
pub mod models;
pub mod parse;
pub mod validate;

pub struct IntegerNewtype<T: IntegerType>(PhantomData<T>);

impl<T> Newtype for IntegerNewtype<T>
where
    T: IntegerType + ToTokens + FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    type Sanitizer = IntegerSanitizer<T>;
    type Validator = IntegerValidator<T>;
    type TypedTrait = IntegerDeriveTrait;

    fn parse_attributes(attrs: TokenStream) -> Result<Attributes<IntegerGuard<T>>, syn::Error> {
        parse::parse_attributes::<T>(attrs)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedItem<DeriveTrait>>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        let has_validation = guard.has_validation();
        validate_integer_derive_traits(derive_traits, has_validation)
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

        let inner_type = T::integer_inner_type();

        gen_nutype_for_integer(
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
