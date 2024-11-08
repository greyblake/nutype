use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};
use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::common::{
    gen::GenerateNewtype,
    models::{Attributes, GenerateParams, Guard, Newtype, SpannedDeriveTrait, TypeName},
};

use self::{
    models::{
        IntegerDeriveTrait, IntegerGuard, IntegerInnerType, IntegerSanitizer, IntegerType,
        IntegerValidator,
    },
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
    type InnerType = IntegerInnerType;

    fn parse_attributes(
        attrs: TokenStream,
        type_name: &TypeName,
    ) -> Result<Attributes<IntegerGuard<T>, SpannedDeriveTrait>, syn::Error> {
        parse::parse_attributes::<T>(attrs, type_name)
    }

    fn validate(
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        derive_traits: Vec<SpannedDeriveTrait>,
    ) -> Result<HashSet<Self::TypedTrait>, syn::Error> {
        let has_validation = guard.has_validation();
        validate_integer_derive_traits(derive_traits, has_validation)
    }

    fn generate(
        params: GenerateParams<
            IntegerInnerType,
            Self::TypedTrait,
            Guard<Self::Sanitizer, Self::Validator>,
        >,
    ) -> Result<TokenStream, syn::Error> {
        IntegerNewtype::gen_nutype(params)
    }
}
