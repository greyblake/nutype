use crate::common::models::Attributes;
use crate::common::models::NewUnchecked;
use crate::common::models::RawGuard;
use crate::common::parse::define_parseable_enum;
use crate::string::models::StringGuard;
use crate::string::models::{StringSanitizer, StringValidator};
use crate::utils::match_feature;
use darling::export::NestedMeta;
use darling::util::Flag;
use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::TokenStream;
use syn::Expr;
use syn::Meta;

use super::models::SpannedStringSanitizers;
use super::models::SpannedStringValidators;
use super::models::{RegexDef, SpannedStringSanitizer, SpannedStringValidator};
use super::validate::validate_string_meta;

#[derive(Debug, FromMeta)]
pub struct ParseableAttributes<S: Default, V: Default> {
    #[darling(default)]
    pub sanitize: S,

    #[darling(default)]
    pub validate: V,

    // NOTE: darling::util::Flag does not allow to obtain Span at the moment, so we use
    // bool here.
    // See https://github.com/TedDriggs/darling/issues/242
    //
    /// `new_unchecked` flag
    #[darling(default)]
    pub new_unchecked: SpannedValue<bool>,

    // NOTE: By default string literals are not parsed as idents, which is not the
    // way it should be identically.
    // See https://github.com/TedDriggs/darling/issues/229
    //
    /// Value for Default trait. Provide with `default = `
    #[darling(with = "preserve_str_literal")]
    pub default: Option<Expr>,
}

pub fn preserve_str_literal(meta: &Meta) -> darling::Result<Option<Expr>> {
    match meta {
        Meta::Path(_) => Err(darling::Error::unsupported_format("path").with_span(meta)),
        Meta::List(_) => Err(darling::Error::unsupported_format("list").with_span(meta)),
        Meta::NameValue(nv) => Ok(Some(nv.value.clone())),
    }
}

type StringParseableAttributes =
    ParseableAttributes<SpannedStringSanitizers, SpannedStringValidators>;

pub fn parse_attributes(stream: TokenStream) -> Result<Attributes<StringGuard>, darling::Error> {
    let items = NestedMeta::parse_meta_list(stream)?;

    let parsed_attrs = StringParseableAttributes::from_list(&items)?;

    let StringParseableAttributes {
        sanitize,
        validate,
        new_unchecked: new_unchecked_flag,
        default: maybe_default_meta,
    } = parsed_attrs;

    let raw_guard = RawGuard {
        sanitizers: sanitize.0,
        validators: validate.0,
    };
    let guard = validate_string_meta(raw_guard)?;

    let new_unchecked = if *new_unchecked_flag.as_ref() {
        match_feature!("new_unchecked",
            on => {
                NewUnchecked::On
            },
            off => {
                // The feature is not enabled, so we return an error
                let msg = "To generate ::new_unchecked() function, the feature `new_unchecked` of crate `nutype` needs to be enabled.\nBut you ought to know: generally, THIS IS A BAD IDEA.\nUse it only when you really need it.";
                return Err(darling::Error::custom(msg).with_span(&new_unchecked_flag.span()))
            }
        )
    } else {
        NewUnchecked::Off
    };

    // Turn the default value into TokenStream
    let maybe_default_value = maybe_default_meta.map(|meta| quote::quote!(#meta));

    Ok(Attributes {
        new_unchecked,
        guard,
        maybe_default_value,
    })
}

impl FromMeta for SpannedStringSanitizers {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut errors = darling::Error::accumulator();

        let raw_sanitizers: Vec<ParseableStringSanitizer> = items
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

        Ok(Self(sanitizers))
    }

    fn from_word() -> darling::Result<Self> {
        // Provide a better error message than the default implementation
        let msg = concat!(
            "`sanitize` must be used with parenthesis.\n",
            "For example:\n",
            "\n",
            "    sanitize(trim)\n\n"
        );
        Err(darling::Error::custom(msg))
    }
}

impl FromMeta for SpannedStringValidators {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut errors = darling::Error::accumulator();

        let parseable_validators: Vec<ParseableStringValidator> = items
            .iter()
            .flat_map(|arg| {
                let res = ParseableStringValidator::from_list(std::slice::from_ref(arg));
                errors.handle(res)
            })
            .collect();

        let validators: Vec<SpannedStringValidator> = parseable_validators
            .into_iter()
            .flat_map(|pv| {
                let res = pv.into_spanned_string_validator();
                errors.handle(res)
            })
            .flatten()
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
            "    validate(not_empty)\n\n"
        );
        Err(darling::Error::custom(msg))
    }
}

define_parseable_enum! {
    parseable_name: ParseableStringSanitizer,
    raw_name: RawStringSanitizer,
    variants: {
        Trim: Flag,
        Lowercase: Flag,
        Uppercase: Flag,
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
            Trim(flag) if flag.is_present() => Some(StringSanitizer::Trim),
            Trim(_) => None,
            Lowercase(flag) if flag.is_present() => Some(StringSanitizer::Lowercase),
            Lowercase(_) => None,
            Uppercase(flag) if flag.is_present() => Some(StringSanitizer::Uppercase),
            Uppercase(_) => None,
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
        NotEmpty: Flag,
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
            NotEmpty(flag) if flag.is_present() => Some(StringValidator::NotEmpty),
            NotEmpty(_) => None,
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
