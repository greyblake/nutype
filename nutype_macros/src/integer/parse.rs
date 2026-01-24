use core::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::common::{
    models::{Attributes, SpannedDeriveTrait, TypeName},
    parse::{
        ParseableAttributes, parse_number_or_expr, parse_sanitizer_kind,
        parse_typed_custom_function, parse_validator_kind,
    },
};
use proc_macro2::TokenStream;
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use super::{
    models::{
        IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerSanitizerKind, IntegerValidator,
        IntegerValidatorKind, SpannedIntegerSanitizer, SpannedIntegerValidator,
    },
    validate::validate_integer_guard,
};

pub fn parse_attributes<T>(
    input: TokenStream,
    type_name: &TypeName,
) -> Result<Attributes<IntegerGuard<T>, SpannedDeriveTrait>, syn::Error>
where
    T: FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    let attrs: ParseableAttributes<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>> =
        syn::parse2(input)?;

    let ParseableAttributes {
        sanitizers,
        validation,
        new_unchecked,
        const_fn,
        constructor_visibility,
        default,
        derive_traits,
        derive_unchecked_traits,
    } = attrs;
    let raw_guard = IntegerRawGuard {
        sanitizers,
        validation,
    };
    let guard = validate_integer_guard(raw_guard, type_name)?;
    Ok(Attributes {
        new_unchecked,
        const_fn,
        constructor_visibility,
        guard,
        default,
        derive_traits,
        derive_unchecked_traits,
    })
}

impl<T> Parse for SpannedIntegerValidator<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, _ident) = parse_validator_kind(input)?;

        match kind {
            IntegerValidatorKind::Greater => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedIntegerValidator {
                    item: IntegerValidator::Greater(number),
                    span,
                })
            }
            IntegerValidatorKind::GreaterOrEqual => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedIntegerValidator {
                    item: IntegerValidator::GreaterOrEqual(number),
                    span,
                })
            }
            IntegerValidatorKind::Less => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedIntegerValidator {
                    item: IntegerValidator::Less(number),
                    span,
                })
            }
            IntegerValidatorKind::LessOrEqual => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedIntegerValidator {
                    item: IntegerValidator::LessOrEqual(number),
                    span,
                })
            }
            IntegerValidatorKind::Predicate => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function::<&T>(input)?;
                Ok(SpannedIntegerValidator {
                    item: IntegerValidator::Predicate(typed_custom_function),
                    span,
                })
            }
        }
    }
}

impl<T> Parse for SpannedIntegerSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, ident) = parse_sanitizer_kind(input)?;

        match kind {
            IntegerSanitizerKind::With => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function::<T>(input)?;
                Ok(SpannedIntegerSanitizer {
                    item: IntegerSanitizer::With(typed_custom_function),
                    span,
                })
            }
            IntegerSanitizerKind::_Phantom => {
                let msg = format!("Unknown validator `{ident}`");
                Err(syn::Error::new(ident.span(), msg))
            }
        }
    }
}
