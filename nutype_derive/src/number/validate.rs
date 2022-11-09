use crate::common::validate::validate_duplicates;

use super::models::{
    NewtypeNumberMeta, NumberSanitizer, NumberValidator, RawNewtypeNumberMeta,
    SpannedNumberSanitizer, SpannedNumberValidator,
};

pub fn validate_number_meta<T>(
    raw_meta: RawNewtypeNumberMeta<T>,
) -> Result<NewtypeNumberMeta<T>, syn::Error>
where
    T: PartialOrd + Clone,
{
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
) -> Result<Vec<NumberValidator<T>>, syn::Error>
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
            NumberValidator::Min(ref min) => Some((v.span, min.clone())),
            _ => None,
        })
        .next();
    let maybe_max = validators
        .iter()
        .flat_map(|v| match v.item {
            NumberValidator::Max(ref max) => Some((v.span, max.clone())),
            _ => None,
        })
        .next();
    if let (Some((min_span, min)), Some((max_span, max))) = (maybe_min, maybe_max) {
        if min > max {
            let msg = "`min` cannot be greater than `max`.\nSometimes we all need a little break.";
            let span = min_span.join(max_span).unwrap();
            let err = syn::Error::new(span, &msg);
            return Err(err);
        }
    }

    let validators: Vec<_> = validators.into_iter().map(|v| v.item).collect();
    Ok(validators)
}

fn validate_sanitizers<T>(
    sanitizers: Vec<SpannedNumberSanitizer<T>>,
) -> Result<Vec<NumberSanitizer<T>>, syn::Error>
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
            NumberSanitizer::Clamp { ref min, ref max } => {
                Some((san.span, (min.clone(), max.clone())))
            } // _ => None,
        })
        .next();
    if let Some((span, (min, max))) = maybe_clamp {
        if min > max {
            let msg = "Min cannot be creater than max in `clamp`";
            let err = syn::Error::new(span, &msg);
            return Err(err);
        }
    }

    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.item).collect();
    Ok(sanitizers)
}
