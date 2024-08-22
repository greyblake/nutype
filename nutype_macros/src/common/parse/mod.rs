pub mod derive_trait;
pub mod meta;

use std::{
    any::type_name,
    fmt::{Debug, Display},
    str::FromStr,
};

use cfg_if::cfg_if;
use kinded::{Kind, Kinded};
use proc_macro2::{Ident, Span};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
    Expr, Lit, Token,
};

use crate::common::models::SpannedDeriveTrait;

use super::models::{
    CustomFunction, ErrorTypeName, NewUnchecked, TypedCustomFunction, ValueOrExpr,
};

pub fn is_doc_attribute(attribute: &syn::Attribute) -> bool {
    match attribute.path().segments.first() {
        Some(path_segment) => path_segment.ident == "doc",
        None => false,
    }
}

pub fn is_derive_attribute(attribute: &syn::Attribute) -> bool {
    match attribute.path().segments.first() {
        Some(path_segment) => path_segment.ident == "derive",
        None => false,
    }
}

pub fn intercept_derive_macro(attributes: &[syn::Attribute]) -> Result<(), syn::Error> {
    // Return an error if attempt to use `#[derive(..)]` is detected.
    for attr in attributes.iter() {
        if is_derive_attribute(attr) {
            let msg = concat!(
                "#[derive(..)] macro is not allowed to be used with #[nutype]. ",
                "If you want to derive traits use `derive(..) attribute within #[nutype] macro:\n\n",
                "    #[nutype(\n",
                "        derive(Debug, Clone, AsRef)\n",
                "    )]\n\n",
            );
            return Err(syn::Error::new(attr.span(), msg));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct ParseableAttributes<Sanitizer, Validator> {
    /// Parsed from `sanitize(...)` attribute
    pub sanitizers: Vec<Sanitizer>,

    /// Parsed from `validate(...)` attribute
    pub validation: Option<RawValidation<Validator>>,

    /// Parsed from `new_unchecked` attribute
    pub new_unchecked: NewUnchecked,

    /// Parsed from `default = ` attribute
    pub default: Option<Expr>,

    /// Parsed from `derive(...)` attribute
    pub derive_traits: Vec<SpannedDeriveTrait>,
}

enum ValidateAttr<Validator: Parse + Kinded> {
    Standard(Validator),
    Extra(ExtraValidateAttr),
}

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
enum ExtraValidateAttr {
    Error(ErrorTypeName),
    With(CustomFunction),
}

impl Parse for ExtraValidateAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, _ident) = parse_validator_kind(input)?;
        match kind {
            ExtraValidateAttrKind::Error => {
                let _eq: Token![=] = input.parse()?;
                let error: ErrorTypeName = input.parse()?;
                Ok(ExtraValidateAttr::Error(error))
            }
            ExtraValidateAttrKind::With => {
                let _eq: Token![=] = input.parse()?;
                let custom_function: CustomFunction = input.parse()?;
                Ok(ExtraValidateAttr::With(custom_function))
            }
        }
    }
}

impl<Validator> Parse for ValidateAttr<Validator>
where
    Validator: Parse + Kinded,
    <Validator as Kinded>::Kind: Kind + Display + 'static,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // NOTE: ParseStream has interior mutability, so we want to try to parse validator,
        // but we don't want to advance the input if it fails.
        if input.fork().parse::<Validator>().is_ok() {
            let validator: Validator = input.parse()?;
            Ok(ValidateAttr::Standard(validator))
        } else if input.fork().parse::<ExtraValidateAttr>().is_ok() {
            let extra_attr: ExtraValidateAttr = input.parse()?;
            Ok(ValidateAttr::Extra(extra_attr))
        } else {
            let possible_values: String = <Validator as Kinded>::Kind::all()
                .iter()
                .map(|k| format!("`{k}`"))
                .filter(|s| s != "`phantom`") // filter out _Phantom variant
                .chain(["`with`", "`error`"].iter().map(|s| s.to_string()))
                .collect::<Vec<_>>()
                .join(", ");
            let ident: Ident = input.parse()?;
            let msg = format!("Unknown validation attribute: `{ident}`.\nPossible attributes are {possible_values}.");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}

#[derive(Debug)]
pub enum RawValidation<Validator> {
    Custom {
        with: CustomFunction,
        error: ErrorTypeName,
    },
    Standard {
        validators: Vec<Validator>,
    },
}

impl<Validator> Parse for RawValidation<Validator>
where
    Validator: Parse + Kinded,
    <Validator as Kinded>::Kind: Kind + Display + 'static,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = input.parse_terminated(ValidateAttr::<Validator>::parse, Token![,])?;
        let attrs: Vec<ValidateAttr<Validator>> = items.into_iter().collect();

        let mut validators: Vec<Validator> = Vec::new();
        let mut maybe_with: Option<CustomFunction> = None;
        let mut maybe_error: Option<ErrorTypeName> = None;

        for attr in attrs {
            match attr {
                ValidateAttr::Standard(validator) => {
                    validators.push(validator);
                }
                ValidateAttr::Extra(extra_attr) => match extra_attr {
                    ExtraValidateAttr::Error(error) => {
                        if maybe_error.is_some() {
                            let msg = "Duplicate `error` attribute";
                            return Err(syn::Error::new(error.span(), msg));
                        }
                        maybe_error = Some(error);
                    }
                    ExtraValidateAttr::With(with) => {
                        if maybe_with.is_some() {
                            let msg = "Duplicate `with` attribute";
                            return Err(syn::Error::new(with.span(), msg));
                        }
                        maybe_with = Some(with);
                    }
                },
            }
        }

        match (validators.len(), maybe_with, maybe_error) {
            (0, Some(with), Some(error)) => Ok(RawValidation::Custom { with, error }),
            (0, Some(_), None) => {
                let msg = "The `with` attribute requires an accompanying `error` attribute.\nPlease provide the error type that the `with` validation function returns.";
                Err(syn::Error::new(input.span(), msg))
            }
            (0, None, Some(error_type)) => {
                let msg = format!("The `error` attribute requires an accompanying `with` attribute.\nPlease provide the validation function that returns Result<(), {error_type}>.");
                Err(syn::Error::new(input.span(), msg))
            }
            (0, None, None) => {
                let msg = "At least one validator must be specified";
                Err(syn::Error::new(input.span(), msg))
            }
            (_, None, None) => Ok(RawValidation::Standard { validators }),
            (_, _, _) => {
                let msg =
                    "`with` and `error` attributes cannot be used mixed with other validators.";
                Err(syn::Error::new(input.span(), msg))
            }
        }
    }
}

// By some reason Default cannot be derived.
impl<Sanitizer, Validator> Default for ParseableAttributes<Sanitizer, Validator> {
    fn default() -> Self {
        Self {
            sanitizers: vec![],
            validation: None,
            new_unchecked: NewUnchecked::Off,
            default: None,
            derive_traits: vec![],
        }
    }
}

impl<Sanitizer, Validator> Parse for ParseableAttributes<Sanitizer, Validator>
where
    Sanitizer: Parse,
    Validator: Parse + Kinded,
    <Validator as Kinded>::Kind: Kind + Display + 'static,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = ParseableAttributes::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if ident == "sanitize" {
                if input.peek(Paren) {
                    let content;
                    parenthesized!(content in input);
                    let items = content.parse_terminated(Sanitizer::parse, Token![,])?;
                    attrs.sanitizers = items.into_iter().collect();
                } else {
                    let msg = concat!(
                        "`sanitize` must be used with parenthesis.\n",
                        "For example:\n\n",
                        "    sanitize(trim)\n\n"
                    );
                    return Err(syn::Error::new(ident.span(), msg));
                }
            } else if ident == "validate" {
                if input.peek(Paren) {
                    let content;
                    parenthesized!(content in input);
                    let validation: RawValidation<Validator> = content.parse()?;
                    attrs.validation = Some(validation);
                } else {
                    let msg = concat!(
                        "`validate` must be used with parenthesis.\n",
                        "For example:\n\n",
                        "    validate(less_or_equal = 99)\n\n"
                    );
                    return Err(syn::Error::new(ident.span(), msg));
                }
            } else if ident == "derive" {
                if input.peek(Paren) {
                    let content;
                    parenthesized!(content in input);
                    let items = content.parse_terminated(SpannedDeriveTrait::parse, Token![,])?;
                    attrs.derive_traits = items.into_iter().collect();
                } else {
                    let msg = concat!(
                        "`derive` must be used with parenthesis.\n",
                        "For example:\n\n",
                        "    derive(Debug, Clone, AsRef)\n\n"
                    );
                    return Err(syn::Error::new(ident.span(), msg));
                }
            } else if ident == "default" {
                let _eq: Token![=] = input.parse()?;
                let default_expr: Expr = input.parse()?;
                attrs.default = Some(default_expr);
            } else if ident == "new_unchecked" {
                cfg_if! {
                    if #[cfg(feature = "new_unchecked")] {
                        attrs.new_unchecked = NewUnchecked::On;
                    } else {
                        // The feature is not enabled, so we return an error
                        let msg = concat!(
                            "To generate ::new_unchecked() function, the feature `new_unchecked` of crate `nutype` needs to be enabled.\n",
                            "But you ought to know: generally, THIS IS A BAD IDEA.\nUse it only when you really need it."
                        );
                        return Err(syn::Error::new(ident.span(), msg));
                    }
                }
            } else {
                let msg = format!("Unknown attribute `{ident}`");
                return Err(syn::Error::new(ident.span(), msg));
            }

            // Parse `,` unless it's the end of the stream
            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        Ok(attrs)
    }
}

pub fn parse_number<T>(input: ParseStream) -> syn::Result<(T, Span)>
where
    T: FromStr,
{
    let mut number_str = String::with_capacity(16);
    if input.peek(Token![-]) {
        let _: Token![-] = input.parse()?;
        number_str.push('-');
    }

    let lit: Lit = input.parse()?;
    let lit_str = match &lit {
        Lit::Float(lf) => lf.to_string(),
        Lit::Int(li) => li.to_string(),
        _ => {
            let msg = "Expected number literal";
            return Err(syn::Error::new(lit.span(), msg));
        }
    };

    number_str.push_str(&lit_str.replace('_', ""));

    let number: T = number_str.parse::<T>().map_err(|_err| {
        let msg = format!("Expected {}, got `{}`", type_name::<T>(), number_str);
        syn::Error::new(lit.span(), msg)
    })?;

    Ok((number, lit.span()))
}

/// Try to parse input as a number of type T (if the value specified directly)
/// If that fails then try to parse it as an expression (if the value is specified as an expression, a constant, etc.)
pub fn parse_number_or_expr<T>(input: ParseStream) -> syn::Result<(ValueOrExpr<T>, Span)>
where
    T: FromStr,
{
    if let Ok((number, span)) = parse_number::<T>(input) {
        Ok((ValueOrExpr::Value(number), span))
    } else {
        let expr: Expr = input.parse()?;
        let span = expr.span();
        Ok((ValueOrExpr::Expr(expr), span))
    }
}

// NOTE: This is a quite hacky way to obtain a syn::Type from `T`.
// Is there a better way?
pub fn parse_typed_custom_function<T>(
    input: ParseStream,
) -> syn::Result<(TypedCustomFunction, Span)> {
    let tp_str = std::any::type_name::<T>();
    parse_typed_custom_function_raw(input, tp_str)
}

pub fn parse_typed_custom_function_raw(
    input: ParseStream,
    tp_str: &'static str,
) -> syn::Result<(TypedCustomFunction, Span)> {
    let custom_function: CustomFunction = input.parse()?;
    let span = custom_function.span();
    let tp: syn::Type = syn::parse_str(tp_str)?;
    let typed_custom_function = custom_function.try_into_typed(&tp)?;
    Ok((typed_custom_function, span))
}

pub fn parse_sanitizer_kind<K>(input: ParseStream) -> syn::Result<(K, Ident)>
where
    K: std::str::FromStr + kinded::Kind + std::fmt::Display + 'static,
{
    parse_kind("sanitizer", input)
}

pub fn parse_validator_kind<K>(input: ParseStream) -> syn::Result<(K, Ident)>
where
    K: std::str::FromStr + kinded::Kind + std::fmt::Display + 'static,
{
    parse_kind("validator", input)
}

/// Parse ident from ParseStream and tries to parse it further into Kind of sanitizer or validator.
/// Build a helpful error on failure.
fn parse_kind<K>(attr_type: &str, input: ParseStream) -> syn::Result<(K, Ident)>
where
    K: std::str::FromStr + kinded::Kind + std::fmt::Display + 'static,
{
    let ident: Ident = input.parse()?;
    let attr_name = ident.to_string();

    if let Ok(kind) = attr_name.parse::<K>() {
        // kinded parses enum variants spelled in different cases (PascalCase, camelCase,
        // snake_case, etc.)
        // Here we want to enforce usage of snake_case only.
        let strict_attr_name = kind.to_string();
        if strict_attr_name == attr_name {
            Ok((kind, ident))
        } else {
            let msg = format!("Unknown {attr_type} `{ident}`. Did you mean `{strict_attr_name}`?");
            Err(syn::Error::new(ident.span(), msg))
        }
    } else {
        let possible_values: String = K::all()
            .iter()
            .map(|k| format!("`{k}`"))
            .filter(|s| s != "`phantom`") // filter out _Phantom variant
            .collect::<Vec<_>>()
            .join(", ");
        let msg = format!("Unknown {attr_type} `{ident}`.\nPossible values are {possible_values}.");
        Err(syn::Error::new(ident.span(), msg))
    }
}
