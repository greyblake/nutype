use crate::{
    common::{
        models::{Attributes, SpannedDeriveTrait, SpannedItem},
        parse::{parse_number, parse_typed_custom_function_raw, ParseableAttributes},
    },
    string::models::{StringGuard, StringRawGuard, StringSanitizer, StringValidator},
};
use cfg_if::cfg_if;
use proc_macro2::{Ident, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    LitStr, Path, Token,
};

use super::{
    models::{RegexDef, SpannedStringSanitizer, SpannedStringValidator},
    validate::validate_string_meta,
};

pub fn parse_attributes(
    input: TokenStream,
) -> Result<Attributes<StringGuard, SpannedDeriveTrait>, syn::Error> {
    let attrs: ParseableAttributes<SpannedStringSanitizer, SpannedStringValidator> =
        syn::parse2(input)?;

    let ParseableAttributes {
        sanitizers,
        validators,
        new_unchecked,
        default,
        derive_traits,
    } = attrs;
    let raw_guard = StringRawGuard {
        sanitizers,
        validators,
    };
    let guard = validate_string_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        default,
        derive_traits,
    })
}

impl Parse for SpannedStringSanitizer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "trim" {
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::Trim,
                span: ident.span(),
            })
        } else if ident == "lowercase" {
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::Lowercase,
                span: ident.span(),
            })
        } else if ident == "uppercase" {
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::Uppercase,
                span: ident.span(),
            })
        } else if ident == "with" {
            let _eq: Token![=] = input.parse()?;
            let (typed_custom_function, span) = parse_typed_custom_function_raw(input, "String")?;
            Ok(SpannedStringSanitizer {
                item: StringSanitizer::With(typed_custom_function),
                span,
            })
        } else {
            let msg = format!("Unknown sanitizer `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
        }
    }
}

impl Parse for SpannedStringValidator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "min_len" {
            let _: Token![=] = input.parse()?;
            let (min_len, span) = parse_number::<usize>(input)?;
            Ok(SpannedStringValidator {
                item: StringValidator::MinLen(min_len),
                span,
            })
        } else if ident == "max_len" {
            let _: Token![=] = input.parse()?;
            let (max_len, span) = parse_number::<usize>(input)?;
            Ok(SpannedStringValidator {
                item: StringValidator::MaxLen(max_len),
                span,
            })
        } else if ident == "not_empty" {
            Ok(SpannedStringValidator {
                item: StringValidator::NotEmpty,
                span: ident.span(),
            })
        } else if ident == "predicate" {
            let _eq: Token![=] = input.parse()?;
            let (typed_custom_function, span) = parse_typed_custom_function_raw(input, "&str")?;
            Ok(SpannedStringValidator {
                item: StringValidator::Predicate(typed_custom_function),
                span,
            })
        } else if ident == "regex" {
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
                        "IMPORTANT: Make sure that your crate EXPLICITLY depends on `regex` and `lazy_static` crates.\n",
                        "And... don't forget to take care of yourself and your beloved ones. That is even more important.",
                    );
                    Err(syn::Error::new(ident.span(), msg))
                }
            }
        } else if ident == "with" {
            let msg = "Deprecated validator `with`. It was renamed to `predicate`";
            Err(syn::Error::new(ident.span(), msg))
        } else {
            let msg = format!("Unknown validator `{ident}`");
            Err(syn::Error::new(ident.span(), msg))
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
