use std::fmt::Debug;
use std::str::FromStr;

use crate::common::{
    models::Attributes,
    parse::{
        define_parseable_enum_t, is_comma, parse_nutype_attributes, parse_with_token_stream,
        split_and_parse,
    },
};
use darling::{export::NestedMeta, util::SpannedValue, Error, FromMeta};
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::Expr;

use super::{
    models::{
        IntegerGuard, IntegerRawGuard, IntegerSanitizer, IntegerValidator, SpannedIntegerSanitizer,
        SpannedIntegerValidator, SpannedIntegerValidators,
    },
    validate::validate_number_meta,
};

pub fn parse_attributes<T>(
    input: TokenStream,
) -> Result<Attributes<IntegerGuard<T>>, darling::Error>
where
    T: FromStr + PartialOrd + Clone + Copy + FromMeta,
    <T as FromStr>::Err: Debug,
{
    let raw_attrs = parse_raw_attributes(input)?;
    let Attributes {
        new_unchecked,
        guard: raw_guard,
        maybe_default_value,
    } = raw_attrs;
    let guard = validate_number_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
    })
}

fn parse_raw_attributes<T>(
    input: TokenStream,
) -> Result<Attributes<IntegerRawGuard<T>>, darling::Error>
where
    T: FromStr + FromMeta + Clone + Copy,
    <T as FromStr>::Err: Debug,
{
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs<T>(
    stream: TokenStream,
) -> Result<Vec<SpannedIntegerSanitizer<T>>, darling::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_sanitize_attr)
}

fn parse_sanitize_attr<T>(
    tokens: Vec<TokenTree>,
) -> Result<SpannedIntegerSanitizer<T>, darling::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut token_iter = tokens.iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "with" => {
                // Preserve the rest as `custom_sanitizer_fn`
                let stream = parse_with_token_stream(token_iter, ident.span())?;
                let span = ident.span();
                let sanitizer = IntegerSanitizer::With(stream);
                Ok(SpannedIntegerSanitizer::new(sanitizer, span))
            }
            unknown_sanitizer => {
                let msg = format!("Unknown sanitizer `{unknown_sanitizer}`");
                let error = syn::Error::new(ident.span(), msg).into();
                Err(error)
            }
        }
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax.").into())
    }
}

fn parse_validate_attrs<T>(stream: TokenStream) -> Result<Vec<SpannedIntegerValidator<T>>, Error>
where
    T: FromStr + FromMeta + Clone + Copy,
    <T as FromStr>::Err: Debug,
{
    let items = NestedMeta::parse_meta_list(stream)?;
    let validators = SpannedIntegerValidators::<T>::from_list(&items)?;

    Ok(validators.0)
}

/*
fn parse_validate_attr<T>(
    tokens: Vec<TokenTree>,
) -> Result<SpannedIntegerValidator<T>, darling::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let mut token_iter = tokens.into_iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        match ident.to_string().as_ref() {
            "min" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = IntegerValidator::Min(value);
                let parsed_validator = SpannedIntegerValidator::new(validator, ident.span());
                Ok(parsed_validator)
            }
            "max" => {
                let (value, _iter) = parse_value_as_number(token_iter)?;
                let validator = IntegerValidator::Max(value);
                let parsed_validator = SpannedIntegerValidator::new(validator, ident.span());
                Ok(parsed_validator)
            }
            "with" => {
                let rest_tokens: Vec<_> = token_iter.collect();
                let stream = parse_with_token_stream(rest_tokens.iter(), ident.span())?;
                let span = ident.span();
                let validator = IntegerValidator::With(stream);
                Ok(SpannedIntegerValidator::new(validator, span))
            }
            unknown_validator => {
                let msg = format!("Unknown validation rule `{unknown_validator}`");
                let error = syn::Error::new(ident.span(), msg).into();
                Err(error)
            }
        }
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax.").into())
    }
}
*/

define_parseable_enum_t! {
    parseable_name: ParseableIntegerValidator,
    raw_name: RawIntegerValidator,
    variants: {
        Min: T,
        Max: T,
        With: Expr,
    }
}

impl<T: Clone + Copy> ParseableIntegerValidator<T> {
    fn into_spanned_validator(self) -> Result<SpannedIntegerValidator<T>, darling::Error> {
        use RawIntegerValidator::*;

        let spanned_raw = self.into_spanned_raw();
        let span = spanned_raw.span();
        let raw = spanned_raw.as_ref();

        let validator = match raw {
            Min(min) => IntegerValidator::Min(*min),
            Max(max) => IntegerValidator::Max(*max),
            With(expr) => IntegerValidator::With(quote::quote!(#expr)),
        };

        Ok(SpannedValue::new(validator, span))
    }
}

impl<T: FromMeta + Clone + Copy> FromMeta for SpannedIntegerValidators<T> {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut errors = darling::Error::accumulator();

        let parseable_validators: Vec<ParseableIntegerValidator<T>> = items
            .iter()
            .flat_map(|arg| {
                let res = ParseableIntegerValidator::<T>::from_list(std::slice::from_ref(arg));
                errors.handle(res)
            })
            .collect();

        let validators: Vec<SpannedIntegerValidator<T>> = parseable_validators
            .into_iter()
            .flat_map(|pv| {
                let res = pv.into_spanned_validator();
                errors.handle(res)
            })
            .collect();

        let validators = errors.finish_with(validators)?;
        Ok(Self(validators))
    }

    fn from_word() -> darling::Result<Self> {
        // Provide a better error message than the default implementation
        let msg = concat!(
            "`validate` must be used with parenthesis.\n",
            "For example:\n",
            "\n",
            "    validate(max = 99)\n\n"
        );
        Err(darling::Error::custom(msg))
    }
}
