use core::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::common::{
    models::{Attributes, ConstFn, SpannedDeriveTrait, TypeName},
    parse::{
        ParseableAttributes, parse_number_or_expr, parse_sanitizer_kind,
        parse_typed_custom_function, parse_validator_kind,
    },
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use super::{
    models::{
        DecimalGuard, DecimalLit, DecimalRawGuard, DecimalSanitizer, DecimalSanitizerKind,
        DecimalValidator, DecimalValidatorKind, SpannedDecimalSanitizer, SpannedDecimalValidator,
    },
    validate::validate_decimal_guard,
};

pub fn parse_attributes<T>(
    input: TokenStream,
    type_name: &TypeName,
) -> Result<Attributes<DecimalGuard<T>, SpannedDeriveTrait>, syn::Error>
where
    T: FromStr + PartialOrd + Clone,
    <T as FromStr>::Err: Debug + Display,
{
    let attrs: ParseableAttributes<SpannedDecimalSanitizer<T>, SpannedDecimalValidator<T>> =
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
        cfg_attr_entries,
    } = attrs;

    // Decision 4: `const_fn` is not supported for Decimal, because Decimal's
    // comparison operators are not `const`. Reject it with a friendly message
    // instead of letting it fail deep in codegen.
    if let ConstFn::Const = const_fn {
        let msg = concat!(
            "`const_fn` is not supported for `rust_decimal::Decimal`, ",
            "because Decimal's comparison operators are not `const`."
        );
        return Err(syn::Error::new(proc_macro2::Span::call_site(), msg));
    }

    // A bare numeric literal `default = 0.5` would be emitted as a float literal,
    // which does not coerce to `Decimal`. Rewrite such literals through
    // `DecimalLit` so they emit a real `Decimal`. Non-literal expressions
    // (a `dec!(...)` macro call, a constant, `Decimal::ZERO`, etc.) are passed
    // through unchanged.
    let default = rewrite_decimal_default(default)?;

    let raw_guard = DecimalRawGuard {
        sanitizers,
        validation,
    };
    let guard = validate_decimal_guard(raw_guard, type_name)?;
    Ok(Attributes {
        new_unchecked,
        const_fn,
        constructor_visibility,
        guard,
        default,
        derive_traits,
        derive_unchecked_traits,
        cfg_attr_entries,
    })
}

/// If `default = ...` is a bare numeric literal (possibly negated), rewrite it
/// into a real `Decimal` expression. Otherwise leave it untouched.
fn rewrite_decimal_default(default: Option<syn::Expr>) -> Result<Option<syn::Expr>, syn::Error> {
    let Some(expr) = default else {
        return Ok(None);
    };

    let Some((number_str, span)) = extract_numeric_literal(&expr) else {
        // Not a bare numeric literal (e.g. `dec!(0.5)`, `Decimal::ZERO`, a
        // constant). It is expected to already be a `Decimal`; pass it through.
        return Ok(Some(expr));
    };

    let decimal_lit = number_str.parse::<DecimalLit>().map_err(|err| {
        syn::Error::new(
            span,
            format!("Invalid decimal default value `{number_str}`: {err}"),
        )
    })?;

    let rewritten: syn::Expr = syn::parse2(decimal_lit.to_token_stream())?;
    Ok(Some(rewritten))
}

/// Extract a numeric literal (int or float, optionally prefixed with a unary
/// minus) as a string suitable for `DecimalLit::from_str`, along with its span.
fn extract_numeric_literal(expr: &syn::Expr) -> Option<(String, proc_macro2::Span)> {
    match expr {
        syn::Expr::Lit(syn::ExprLit { lit, .. }) => lit_to_number_string(lit),
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => {
            if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = expr.as_ref() {
                lit_to_number_string(lit).map(|(s, span)| (format!("-{s}"), span))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn lit_to_number_string(lit: &syn::Lit) -> Option<(String, proc_macro2::Span)> {
    match lit {
        syn::Lit::Int(li) => Some((li.to_string().replace('_', ""), li.span())),
        syn::Lit::Float(lf) => Some((lf.to_string().replace('_', ""), lf.span())),
        _ => None,
    }
}

impl<T> Parse for SpannedDecimalValidator<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, _ident) = parse_validator_kind(input)?;

        match kind {
            DecimalValidatorKind::Greater => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedDecimalValidator {
                    item: DecimalValidator::Greater(number),
                    span,
                })
            }
            DecimalValidatorKind::GreaterOrEqual => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedDecimalValidator {
                    item: DecimalValidator::GreaterOrEqual(number),
                    span,
                })
            }
            DecimalValidatorKind::Less => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedDecimalValidator {
                    item: DecimalValidator::Less(number),
                    span,
                })
            }
            DecimalValidatorKind::LessOrEqual => {
                let _eq: Token![=] = input.parse()?;
                let (number, span) = parse_number_or_expr::<T>(input)?;
                Ok(SpannedDecimalValidator {
                    item: DecimalValidator::LessOrEqual(number),
                    span,
                })
            }
            DecimalValidatorKind::Predicate => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function::<&T>(input)?;
                Ok(SpannedDecimalValidator {
                    item: DecimalValidator::Predicate(typed_custom_function),
                    span,
                })
            }
        }
    }
}

impl<T> Parse for SpannedDecimalSanitizer<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, ident) = parse_sanitizer_kind(input)?;

        match kind {
            DecimalSanitizerKind::With => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function::<T>(input)?;
                Ok(SpannedDecimalSanitizer {
                    item: DecimalSanitizer::With(typed_custom_function),
                    span,
                })
            }
            DecimalSanitizerKind::_Phantom => {
                let msg = format!("Unknown sanitizer `{ident}`");
                Err(syn::Error::new(ident.span(), msg))
            }
        }
    }
}
