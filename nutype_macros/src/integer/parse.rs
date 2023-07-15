use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::common::models::{Attributes, CustomFunction};
use crate::common::parse::{parse_number, ParseableAttributes};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Token;

use super::{
    models::{
        IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerValidator, SpannedIntegerSanitizer,
        SpannedIntegerValidator,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(input: TokenStream) -> Result<Attributes<IntegerGuard<T>>, syn::Error>
where
    T: FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    let attrs: ParseableAttributes<SpannedIntegerSanitizer<T>, SpannedIntegerValidator<T>> =
        syn::parse2(input)?;

    let ParseableAttributes {
        sanitizers,
        validators,
        new_unchecked,
        default,
    } = attrs;
    let maybe_default_value = default.map(|expr| quote!(#expr));
    let raw_guard = IntegerRawGuard {
        sanitizers,
        validators,
    };
    let guard = validate_number_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
    })
}

impl<T> Parse for SpannedIntegerValidator<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "min" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_number::<T>(input)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::Min(number),
                span,
            })
        } else if ident == "max" {
            let _eq: Token![=] = input.parse()?;
            let (number, span) = parse_number::<T>(input)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::Max(number),
                span,
            })
        } else if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let custom_function: CustomFunction = input.parse()?;
            let span = custom_function.span();
            // NOTE: this is a hacky way to convert `T` into `syn::Type`
            // Is there a better way?
            let tp_str = format!("&{}", std::any::type_name::<T>());
            let tp: syn::Type = syn::parse_str(&tp_str)?;
            let typed_custom_function = custom_function.try_into_typed(&tp)?;
            Ok(SpannedIntegerValidator {
                item: IntegerValidator::With(typed_custom_function),
                span,
            })
        } else {
            let msg = format!("Unknown validator `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}

impl<T> Parse for SpannedIntegerSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let custom_function: CustomFunction = input.parse()?;
            let span = custom_function.span();
            // NOTE: this is a hacky way to convert `T` into `syn::Type`
            // Is there a better way?
            let tp_str = std::any::type_name::<T>();
            let tp: syn::Type = syn::parse_str(tp_str)?;
            let typed_custom_function = custom_function.try_into_typed(&tp)?;
            Ok(SpannedIntegerSanitizer {
                item: IntegerSanitizer::With(typed_custom_function),
                span,
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}
