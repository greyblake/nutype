use crate::{
    common::{
        models::{Attributes, SpannedDeriveTrait, SpannedItem, TypeName},
        parse::{
            ParseableAttributes, parse_number_or_expr, parse_sanitizer_kind,
            parse_typed_custom_function_raw, parse_validator_kind,
        },
    },
    string::models::{StringGuard, StringRawGuard, StringSanitizer, StringValidator},
};
use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use syn::{
    LitStr, Path, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use super::{
    models::{
        RegexDef, SpannedStringSanitizer, SpannedStringValidator, StringSanitizerKind,
        StringValidatorKind,
    },
    validate::validate_string_guard,
};

pub fn parse_attributes(
    input: TokenStream,
    type_name: &TypeName,
) -> Result<Attributes<StringGuard, SpannedDeriveTrait>, syn::Error> {
    let attrs: ParseableAttributes<SpannedStringSanitizer, SpannedStringValidator> =
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
    let raw_guard = StringRawGuard {
        sanitizers,
        validation,
    };
    let guard = validate_string_guard(raw_guard, type_name)?;
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

impl Parse for SpannedStringSanitizer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, ident) = parse_sanitizer_kind(input)?;

        match kind {
            StringSanitizerKind::Trim => Ok(SpannedStringSanitizer {
                item: StringSanitizer::Trim,
                span: ident.span(),
            }),
            StringSanitizerKind::Lowercase => Ok(SpannedStringSanitizer {
                item: StringSanitizer::Lowercase,
                span: ident.span(),
            }),
            StringSanitizerKind::Uppercase => Ok(SpannedStringSanitizer {
                item: StringSanitizer::Uppercase,
                span: ident.span(),
            }),
            StringSanitizerKind::With => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) =
                    parse_typed_custom_function_raw(input, "String")?;
                Ok(SpannedStringSanitizer {
                    item: StringSanitizer::With(typed_custom_function),
                    span,
                })
            }
        }
    }
}

impl Parse for SpannedStringValidator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (kind, ident) = parse_validator_kind(input)?;

        match kind {
            StringValidatorKind::LenCharMin => {
                let _: Token![=] = input.parse()?;
                let (min_len, span) = parse_number_or_expr::<usize>(input)?;
                Ok(SpannedStringValidator {
                    item: StringValidator::LenCharMin(min_len),
                    span,
                })
            }
            StringValidatorKind::LenCharMax => {
                let _: Token![=] = input.parse()?;
                let (max_len, span) = parse_number_or_expr::<usize>(input)?;
                Ok(SpannedStringValidator {
                    item: StringValidator::LenCharMax(max_len),
                    span,
                })
            }
            StringValidatorKind::NotEmpty => Ok(SpannedStringValidator {
                item: StringValidator::NotEmpty,
                span: ident.span(),
            }),
            StringValidatorKind::Predicate => {
                let _eq: Token![=] = input.parse()?;
                let (typed_custom_function, span) = parse_typed_custom_function_raw(input, "&str")?;
                Ok(SpannedStringValidator {
                    item: StringValidator::Predicate(typed_custom_function),
                    span,
                })
            }
            StringValidatorKind::Regex => {
                cfg_if! {
                    if #[cfg(feature = "regex")] {
                        let _eq: Token![=] = input.parse()?;
                        let SpannedRegexDef {
                            item: regex_def,
                            span,
                        } = input.parse()?;
                        Ok(SpannedStringValidator {
                            item: StringValidator::Regex(regex_def),
                            span
                        })
                    } else {
                        let msg = concat!(
                            "To validate string types with regex, the feature `regex` of the crate `nutype` must be enabled.\n",
                            "IMPORTANT: Make sure that your crate EXPLICITLY depends on the `regex` crate.\n",
                            "And... don't forget to take care of yourself and your beloved ones. That is even more important.",
                        );
                        Err(syn::Error::new(ident.span(), msg))
                    }
                }
            }
        }
    }
}

type SpannedRegexDef = SpannedItem<RegexDef>;

impl Parse for SpannedRegexDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(lit_str) = input.parse::<LitStr>() {
            Ok(SpannedRegexDef {
                span: lit_str.span(),
                item: RegexDef::StringLiteral(lit_str),
            })
        } else if let Ok(path) = input.parse::<Path>() {
            Ok(SpannedRegexDef {
                span: path.span(),
                item: RegexDef::Path(path),
            })
        } else {
            let msg = "regex must be either a string or an ident that refers to a Regex constant";
            Err(syn::Error::new(input.span(), msg))
        }
    }
}
