use std::collections::HashSet;

use proc_macro2::Span;

use crate::{
    common::validate::validate_duplicates,
    models::{DeriveTrait, NormalDeriveTrait, SpannedDeriveTrait},
};

use super::models::{
    IntegerDeriveTrait, IntegerSanitizer, IntegerValidator, NewtypeIntegerMeta,
    RawNewtypeIntegerMeta, SpannedIntegerSanitizer, SpannedIntegerValidator,
};

pub fn validate_number_meta<T>(
    raw_meta: RawNewtypeIntegerMeta<T>,
) -> Result<NewtypeIntegerMeta<T>, syn::Error>
where
    T: PartialOrd + Clone,
{
    let RawNewtypeIntegerMeta {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(NewtypeIntegerMeta::From { sanitizers })
    } else {
        Ok(NewtypeIntegerMeta::TryFrom {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators<T>(
    validators: Vec<SpannedIntegerValidator<T>>,
) -> Result<Vec<IntegerValidator<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validator `{kind}`.\nYou're a great engineer, but don't forget to take care of yourself!")
    })?;

    // max VS min
    let maybe_min = validators
        .iter()
        .flat_map(|v| match &v.item {
            IntegerValidator::Min(ref min) => Some((v.span, min.clone())),
            _ => None,
        })
        .next();
    let maybe_max = validators
        .iter()
        .flat_map(|v| match v.item {
            IntegerValidator::Max(ref max) => Some((v.span, max.clone())),
            _ => None,
        })
        .next();
    if let (Some((_min_span, min)), Some((max_span, max))) = (maybe_min, maybe_max) {
        if min > max {
            let msg = "`min` cannot be greater than `max`.\nSometimes we all need a little break.";
            let err = syn::Error::new(max_span, msg);
            return Err(err);
        }
    }

    let validators: Vec<_> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers<T>(
    sanitizers: Vec<SpannedIntegerSanitizer<T>>,
) -> Result<Vec<IntegerSanitizer<T>>, syn::Error>
where
    T: PartialOrd + Clone,
{
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nIt happens, don't worry. We still love you!")
    })?;

    // Validate Clamp (min VS max)
    let maybe_clamp = sanitizers
        .iter()
        .flat_map(|san| match &san.item {
            IntegerSanitizer::Clamp { ref min, ref max } => {
                Some((san.span, (min.clone(), max.clone())))
            }
            _ => None,
        })
        .next();
    if let Some((span, (min, max))) = maybe_clamp {
        if min > max {
            let msg = "Min cannot be creater than max in `clamp`";
            let err = syn::Error::new(span, msg);
            return Err(err);
        }
    }

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}

pub fn validate_integer_derive_traits(
    spanned_derive_traits: Vec<SpannedDeriveTrait>,
    has_validation: bool,
) -> Result<HashSet<IntegerDeriveTrait>, syn::Error> {
    let mut traits = HashSet::with_capacity(24);

    for spanned_trait in spanned_derive_traits {
        match spanned_trait.item {
            DeriveTrait::Asterisk => {
                traits.extend(unfold_asterisk_traits(has_validation));
            }
            DeriveTrait::Normal(normal_trait) => {
                let string_derive_trait =
                    to_integer_derive_trait(normal_trait, has_validation, spanned_trait.span)?;
                traits.insert(string_derive_trait);
            }
        };
    }

    Ok(traits)
}

fn unfold_asterisk_traits(has_validation: bool) -> impl Iterator<Item = IntegerDeriveTrait> {
    let from_or_try_from = if has_validation {
        IntegerDeriveTrait::TryFrom
    } else {
        IntegerDeriveTrait::From
    };

    [
        from_or_try_from,
        IntegerDeriveTrait::Debug,
        IntegerDeriveTrait::Clone,
        IntegerDeriveTrait::Copy,
        IntegerDeriveTrait::PartialEq,
        IntegerDeriveTrait::Eq,
        IntegerDeriveTrait::PartialOrd,
        IntegerDeriveTrait::Ord,
        IntegerDeriveTrait::FromStr,
        IntegerDeriveTrait::AsRef,
        IntegerDeriveTrait::Hash,
        // TODO: should depend on features
        //
        // IntegerDeriveTrait::Serialize,
        // IntegerDeriveTrait::Deserialize,
        // IntegerDeriveTrait::Arbitrary,
    ]
    .into_iter()
}

fn to_integer_derive_trait(
    tr: NormalDeriveTrait,
    has_validation: bool,
    span: Span,
) -> Result<IntegerDeriveTrait, syn::Error> {
    match tr {
        NormalDeriveTrait::Debug => Ok(IntegerDeriveTrait::Debug),
        NormalDeriveTrait::Clone => Ok(IntegerDeriveTrait::Clone),
        NormalDeriveTrait::PartialEq => Ok(IntegerDeriveTrait::PartialEq),
        NormalDeriveTrait::Eq => Ok(IntegerDeriveTrait::Eq),
        NormalDeriveTrait::PartialOrd => Ok(IntegerDeriveTrait::PartialOrd),
        NormalDeriveTrait::Ord => Ok(IntegerDeriveTrait::Ord),
        NormalDeriveTrait::Into => todo!(),
        NormalDeriveTrait::FromStr => Ok(IntegerDeriveTrait::FromStr),
        NormalDeriveTrait::AsRef => Ok(IntegerDeriveTrait::AsRef),
        NormalDeriveTrait::Hash => Ok(IntegerDeriveTrait::Hash),
        NormalDeriveTrait::Borrow => Ok(IntegerDeriveTrait::Borrow),
        NormalDeriveTrait::Serialize => {
            unimplemented!("Serialize is not yet implemented");
            // traits.insert(IntegerDeriveTrait::Serialize)
        }
        NormalDeriveTrait::Deserialize => {
            unimplemented!("Deserialize is not yet implemented");
            // traits.insert(IntegerDeriveTrait::Deserialize)
        }
        NormalDeriveTrait::Arbitrary => {
            unimplemented!("Arbitrary is not yet implemented");
            // traits.insert(IntegerDeriveTrait::Arbitrary)
        }
        NormalDeriveTrait::Copy => Err(syn::Error::new(
            span,
            "Copy trait cannot be derived for a String based type",
        )),
        NormalDeriveTrait::From => {
            if has_validation {
                Err(syn::Error::new(span, "#[nutype] cannot derive `From` trait, because there is validation defined. Use `TryFrom` instead."))
            } else {
                Ok(IntegerDeriveTrait::From)
            }
        }
        NormalDeriveTrait::TryFrom => {
            if has_validation {
                Ok(IntegerDeriveTrait::TryFrom)
            } else {
                Err(syn::Error::new(span, "#[nutype] cannot derive `TryFrom`, because there is no validation. Use `From` instead."))
            }
        }
    }
}
