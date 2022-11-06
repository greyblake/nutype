use crate::common::validate::validate_duplicates;

use super::models::{
    NewtypeNumberMeta, NumberSanitizer, NumberValidator, RawNewtypeNumberMeta,
    SpannedNumberSanitizer, SpannedNumberValidator,
};

pub fn validate_number_meta<T>(
    raw_meta: RawNewtypeNumberMeta<T>,
) -> Result<NewtypeNumberMeta<T>, Vec<syn::Error>> {
    let RawNewtypeNumberMeta {
        sanitizers,
        validators,
    } = raw_meta;

    let validators = validate_validators(validators)?;
    let sanitizers = validate_sanitizers(sanitizers)?;

    if validators.is_empty() {
        Ok(NewtypeNumberMeta::From { sanitizers })
    } else {
        Ok(NewtypeNumberMeta::TryFrom {
            sanitizers,
            validators,
        })
    }
}

fn validate_validators<T>(
    validators: Vec<SpannedNumberValidator<T>>,
) -> Result<Vec<NumberValidator<T>>, Vec<syn::Error>> {
    validate_duplicates(&validators, |kind| {
        format!("Duplicated validator `{kind}`.\nYou're a great engineer, but don't forget to take care of yourself!")
    })?;

    let validators: Vec<_> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers<T>(
    sanitizers: Vec<SpannedNumberSanitizer<T>>,
) -> Result<Vec<NumberSanitizer<T>>, Vec<syn::Error>> {
    validate_duplicates(&sanitizers, |kind| {
        format!("Duplicated sanitizer `{kind}`.\nIt happens, don't worry. We still love you!")
    })?;

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}
