use std::collections::HashSet;

use proc_macro2::Span;

use crate::common::models::Kind;
use crate::common::models::{DeriveTrait, NormalDeriveTrait, SpannedDeriveTrait};
use crate::common::validate::validate_duplicates;
use crate::string::models::{StringGuard, StringRawGuard, StringSanitizer, StringValidator};

use super::models::{
    SpannedStringSanitizer, SpannedStringValidator, StringDeriveTrait, StringSanitizerKind,
};

pub fn validate_string_meta(raw_meta: StringRawGuard) -> Result<StringGuard, syn::Error> {
    let StringRawGuard {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(StringGuard::WithoutValidation { sanitizers })
    } else {
        Ok(StringGuard::WithValidation {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators(
    validators: Vec<SpannedStringValidator>,
) -> Result<Vec<StringValidator>, syn::Error> {
    // Check duplicates
    //
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validators `{kind}`.\nDon't worry, you still remain ingenious!")
    })?;

    // max_len VS min_len
    //
    let maybe_min_len = validators
        .iter()
        .flat_map(|v| match v.item {
            StringValidator::MinLen(len) => Some((v.span, len)),
            _ => None,
        })
        .next();
    let maybe_max_len = validators
        .iter()
        .flat_map(|v| match v.item {
            StringValidator::MaxLen(len) => Some((v.span, len)),
            _ => None,
        })
        .next();
    if let (Some((_min_len_span, min_len)), Some((max_len_span, max_len))) =
        (maybe_min_len, maybe_max_len)
    {
        if min_len > max_len {
            let msg = "min_len cannot be greater than max_len.\nDon't you find this obvious?";
            let span = max_len_span;
            let err = syn::Error::new(span, msg);
            return Err(err);
        }
    }

    // Validate regex
    //
    #[cfg(feature = "regex")]
    for v in validators.iter() {
        if let StringValidator::Regex(ref regex_def) = v.item {
            regex_validation::validate_regex_def(regex_def, v.span)?;
        }
    }

    let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers(
    sanitizers: Vec<SpannedStringSanitizer>,
) -> Result<Vec<StringSanitizer>, syn::Error> {
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nYou're doing well, it's not that bad unless you forgot to call your mom!")
    })?;

    // Validate lowercase VS uppercase
    let lowercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Lowercase);
    let uppercase = sanitizers
        .iter()
        .find(|&s| s.kind() == StringSanitizerKind::Uppercase);
    if let (Some(lowercase), Some(uppercase)) = (lowercase, uppercase) {
        let msg = format!("Using both sanitizers `{}` and `{}` makes no sense.\nYou're a great developer! Take care of yourself, a 5 mins break may help.", lowercase.kind(), uppercase.kind());
        let span = lowercase.span;
        let err = syn::Error::new(span, msg);
        return Err(err);
    }

    let sanitizers: Vec<StringSanitizer> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_string_derive_traits(
    meta: &StringGuard,
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
) -> Result<HashSet<StringDeriveTrait>, syn::Error> {
    let mut traits = HashSet::with_capacity(24);
    let has_validation = meta.has_validation();

    for spanned_trait in spanned_derive_traits {
        match spanned_trait.item {
            DeriveTrait::Asterisk => {
                traits.extend(unfold_asterisk_traits(has_validation));
            }
            DeriveTrait::Normal(normal_trait) => {
                let string_derive_trait =
                    to_string_derive_trait(normal_trait, has_validation, spanned_trait.span)?;
                traits.insert(string_derive_trait);
            }
        };
    }

    Ok(traits)
}

fn unfold_asterisk_traits(has_validation: bool) -> impl Iterator<Item = StringDeriveTrait> {
    let from_or_try_from = if has_validation {
        StringDeriveTrait::TryFrom
    } else {
        StringDeriveTrait::From
    };

    [
        from_or_try_from,
        StringDeriveTrait::Debug,
        StringDeriveTrait::Clone,
        StringDeriveTrait::PartialEq,
        StringDeriveTrait::Eq,
        StringDeriveTrait::PartialOrd,
        StringDeriveTrait::Ord,
        StringDeriveTrait::FromStr,
        StringDeriveTrait::AsRef,
        StringDeriveTrait::Hash,
    ]
    .into_iter()
}

fn to_string_derive_trait(
    tr: NormalDeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<StringDeriveTrait, syn::Error> {
    match tr {
        NormalDeriveTrait::Debug => Ok(StringDeriveTrait::Debug),
        NormalDeriveTrait::Display => Ok(StringDeriveTrait::Display),
        NormalDeriveTrait::Clone => Ok(StringDeriveTrait::Clone),
        NormalDeriveTrait::PartialEq => Ok(StringDeriveTrait::PartialEq),
        NormalDeriveTrait::Eq => Ok(StringDeriveTrait::Eq),
        NormalDeriveTrait::PartialOrd => Ok(StringDeriveTrait::PartialOrd),
        NormalDeriveTrait::Ord => Ok(StringDeriveTrait::Ord),
        NormalDeriveTrait::FromStr => Ok(StringDeriveTrait::FromStr),
        NormalDeriveTrait::AsRef => Ok(StringDeriveTrait::AsRef),
        NormalDeriveTrait::Hash => Ok(StringDeriveTrait::Hash),
        NormalDeriveTrait::Borrow => Ok(StringDeriveTrait::Borrow),
        NormalDeriveTrait::Into => Ok(StringDeriveTrait::Into),
        NormalDeriveTrait::SerdeSerialize => Ok(StringDeriveTrait::SerdeSerialize),
        NormalDeriveTrait::SerdeDeserialize => Ok(StringDeriveTrait::SerdeDeserialize),
        NormalDeriveTrait::SchemarsJsonSchema => Ok(StringDeriveTrait::SchemarsJsonSchema),
        NormalDeriveTrait::Copy => Err(syn::Error::new(
            span,
            "Copy trait cannot be derived for a String based type",
        )),
        NormalDeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(span, "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead."))
            } else {
                Ok(StringDeriveTrait::From)
            }
        }
        NormalDeriveTrait::TryFrom => Ok(StringDeriveTrait::TryFrom),
    }
}

#[cfg(feature = "regex")]
mod regex_validation {
    use super::*;
    use crate::string::models::RegexDef;
    use proc_macro2::Literal;
    use std::collections::VecDeque;

    pub fn validate_regex_def(regex_def: &RegexDef, span: Span) -> Result<(), syn::Error> {
        match regex_def {
            RegexDef::StringLiteral(lit) => {
                // Try to validate regex at compile time if it's a string literal
                let regex_str = literal_to_string(lit)?;
                match regex::Regex::new(&regex_str) {
                    Ok(_re) => Ok(()),
                    Err(err) => Err(syn::Error::new(span, format!("{err}"))),
                }
            }
            RegexDef::Ident(_) => Ok(()),
        }
    }

    // TODO: write unit tests
    fn literal_to_string(lit: &Literal) -> Result<String, syn::Error> {
        let value = lit.to_string();
        if let Some(s) = unquote_double_quotes(&value) {
            Ok(s)
        } else if let Some(s) = unoquote_inline_doc(&value) {
            Ok(s)
        } else {
            let msg = format!("Could not obtain regex string from the literal: {lit}");
            Err(syn::Error::new(lit.span(), msg))
        }
    }

    // Input:
    //     "^bar{2}$"
    // Output:
    //     ^bar{2}$
    // TODO: write unit tests
    fn unquote_double_quotes(input: &str) -> Option<String> {
        let len = input.len();
        if len > 1 && input.starts_with('"') && input.ends_with('"') {
            let output = input[1..(len - 1)].to_string();
            Some(output)
        } else {
            None
        }
    }

    // Input:
    //     r##"^bar{2}$"##
    // Output:
    //     ^bar{2}$
    // TODO: write unit tests
    fn unoquote_inline_doc(input: &str) -> Option<String> {
        let mut chars: VecDeque<char> = input.chars().collect();

        chars.pop_front(); // get rid of `r`
        while let Some(front_ch) = chars.pop_front() {
            chars.pop_back();
            if front_ch == '"' {
                let output: String = chars.into_iter().collect();
                return Some(output);
            }
        }
        None
    }
}
