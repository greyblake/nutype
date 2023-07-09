use crate::common::models::Attributes;
use crate::common::parse::define_parseable_enum;
use crate::common::parse::parse_nutype_attributes;
use crate::string::models::StringGuard;
use crate::string::models::StringRawGuard;
use crate::string::models::{StringSanitizer, StringValidator};
use crate::utils::match_feature;
use darling::export::NestedMeta;
use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::TokenStream;
use syn::Expr;

use super::models::{RegexDef, SpannedStringSanitizer, SpannedStringValidator};
use super::validate::validate_string_meta;

pub fn parse_attributes(input: TokenStream) -> Result<Attributes<StringGuard>, darling::Error> {
    let raw_attrs = parse_raw_attributes(input)?;
    let Attributes {
        new_unchecked,
        guard: raw_guard,
        maybe_default_value,
    } = raw_attrs;
    let guard = validate_string_meta(raw_guard)?;
    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
    })
}

fn parse_raw_attributes(input: TokenStream) -> Result<Attributes<StringRawGuard>, darling::Error> {
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs(
    stream: TokenStream,
) -> Result<Vec<SpannedStringSanitizer>, darling::Error> {
    let attr_args = NestedMeta::parse_meta_list(stream)?;

    let mut errors = darling::Error::accumulator();

    let raw_sanitizers: Vec<ParseableStringSanitizer> = attr_args
        .iter()
        .flat_map(|arg| {
            let res = ParseableStringSanitizer::from_list(std::slice::from_ref(arg));
            errors.handle(res)
        })
        .collect();

    let raw_sanitizers = errors.finish_with(raw_sanitizers)?;

    let sanitizers: Vec<SpannedStringSanitizer> = raw_sanitizers
        .into_iter()
        .flat_map(ParseableStringSanitizer::into_spanned_string_sanitizer)
        .collect();

    Ok(sanitizers)
}

fn parse_validate_attrs(
    stream: TokenStream,
) -> Result<Vec<SpannedStringValidator>, darling::Error> {
    let attr_args = NestedMeta::parse_meta_list(stream)?;

    let mut errors = darling::Error::accumulator();

    let parseable_validators: Vec<ParseableStringValidator> = attr_args
        .iter()
        .flat_map(|arg| {
            let res = ParseableStringValidator::from_list(std::slice::from_ref(arg));
            errors.handle(res)
        })
        .collect();

    let parseable_validators = errors.finish_with(parseable_validators)?;

    let validators = parseable_validators
        .into_iter()
        .map(ParseableStringValidator::into_spanned_string_validator)
        .collect::<Result<Vec<Option<SpannedStringValidator>>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<SpannedStringValidator>>();

    Ok(validators)
}

define_parseable_enum! {
    parseable_name: ParseableStringSanitizer,
    raw_name: RawStringSanitizer,
    variants: {
        Trim: bool,
        Lowercase: bool,
        Uppercase: bool,
        With: Expr,
    }
}

impl ParseableStringSanitizer {
    fn into_spanned_string_sanitizer(self) -> Option<SpannedStringSanitizer> {
        use RawStringSanitizer::*;

        let spanned_raw = self.into_spanned_raw();
        let span = spanned_raw.span();
        let raw = spanned_raw.as_ref();

        let maybe_sanitizer = match raw {
            Trim(true) => Some(StringSanitizer::Trim),
            Trim(false) => None,
            Lowercase(true) => Some(StringSanitizer::Lowercase),
            Lowercase(false) => None,
            Uppercase(true) => Some(StringSanitizer::Uppercase),
            Uppercase(false) => None,
            With(expr) => Some(StringSanitizer::With(quote::quote!(#expr))),
        };

        maybe_sanitizer.map(|san| SpannedValue::new(san, span))
    }
}

define_parseable_enum! {
    parseable_name: ParseableStringValidator,
    raw_name: RawStringValidator,
    variants: {
        MinLen: usize,
        MaxLen: usize,
        NotEmpty: bool,
        With: Expr,
        Regex: RegexDef,
    }
}

impl ParseableStringValidator {
    fn into_spanned_string_validator(
        self,
    ) -> Result<Option<SpannedStringValidator>, darling::Error> {
        use RawStringValidator::*;

        let spanned_raw = self.into_spanned_raw();
        let span = spanned_raw.span();
        let raw = spanned_raw.as_ref();

        let maybe_validator = match raw {
            MinLen(min) => Some(StringValidator::MinLen(*min)),
            MaxLen(max) => Some(StringValidator::MaxLen(*max)),
            NotEmpty(true) => Some(StringValidator::NotEmpty),
            NotEmpty(false) => None,
            With(expr) => Some(StringValidator::With(quote::quote!(#expr))),
            Regex(regex_def) => {
                match_feature!("regex",
                    on => {
                        Some(StringValidator::Regex(regex_def.clone()))
                    },
                    off => {
                        let msg = concat!(
                            "To validate string types with regex, the feature `regex` of the crate `nutype` must be enabled.\n",
                            "IMPORTANT: Make sure that your crate EXPLICITLY depends on `regex` and `lazy_static` crates.\n",
                            "And... don't forget to take care of yourself and your beloved ones. That is even more important.",
                        );
                        return Err(syn::Error::new(span, msg).into());
                    }
                )
            }
        };
        Ok(maybe_validator.map(|san| SpannedValue::new(san, span)))
    }
}

impl FromMeta for RegexDef {
    fn from_meta(item: &syn::Meta) -> Result<Self, darling::Error> {
        use syn::spanned::Spanned;

        let build_err = || {
            let msg = "regex must be either a string or an ident that refers to a Regex constant";
            darling::Error::from(syn::Error::new(item.span(), msg))
        };

        match syn::LitStr::from_meta(item) {
            Ok(lit) => Ok(Self::StringLiteral(lit)),
            Err(_) => {
                // NOTE: by some reason
                //     syn::Path::from_meta(item)
                // is not getting Path, so we have to do it on our own.
                match item {
                    syn::Meta::NameValue(name_value) => match &name_value.value {
                        syn::Expr::Path(expr_path) => {
                            let path = expr_path.path.to_owned();
                            Ok(Self::Path(path))
                        }
                        _ => Err(build_err()),
                    },
                    _ => Err(build_err()),
                }
            }
        }
    }
}
