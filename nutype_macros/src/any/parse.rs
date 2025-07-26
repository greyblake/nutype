use crate::common::{
    models::{Attributes, CustomFunction, SpannedDeriveTrait, TypeName},
    parse::{ParseableAttributes, parse_sanitizer_kind, parse_validator_kind},
};
use proc_macro2::TokenStream;
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use super::{
    models::{
        AnyGuard, AnyRawGuard, AnySanitizer, AnySanitizerKind, AnyValidator, AnyValidatorKind,
        SpannedAnySanitizer, SpannedAnyValidator,
    },
    validate::validate_any_guard,
};

pub fn parse_attributes(
    input: TokenStream,
    type_name: &TypeName,
) -> Result<Attributes<AnyGuard, SpannedDeriveTrait>, syn::Error> {
    let attrs: ParseableAttributes<SpannedAnySanitizer, SpannedAnyValidator> = syn::parse2(input)?;

    let ParseableAttributes {
        sanitizers,
        validation,
        new_unchecked,
        const_fn,
        default,
        derive_traits,
    } = attrs;
    let raw_guard = AnyRawGuard {
        sanitizers,
        validation,
    };
    let guard = validate_any_guard(raw_guard, type_name)?;
    Ok(Attributes {
        new_unchecked,
        const_fn,
        guard,
        default,
        derive_traits,
    })
}

impl Parse for SpannedAnySanitizer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, _ident) = parse_sanitizer_kind(input)?;

        match kind {
            AnySanitizerKind::With => {
                let _eq: Token![=] = input.parse()?;
                let span = input.span();
                let custom_function: CustomFunction = input.parse()?;
                Ok(SpannedAnySanitizer {
                    item: AnySanitizer::With(custom_function),
                    span,
                })
            }
        }
    }
}

impl Parse for SpannedAnyValidator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, _ident) = parse_validator_kind(input)?;

        match kind {
            AnyValidatorKind::Predicate => {
                let _eq: Token![=] = input.parse()?;
                let span = input.span();
                let custom_function: CustomFunction = input.parse()?;
                Ok(SpannedAnyValidator {
                    item: AnyValidator::Predicate(custom_function),
                    span,
                })
            }
        }
    }
}
