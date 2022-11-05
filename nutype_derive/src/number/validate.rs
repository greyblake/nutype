use super::models::{
    NewtypeNumberMeta, NumberSanitizer, NumberValidator, ParsedNumberSanitizer,
    ParsedNumberValidator, RawNewtypeNumberMeta,
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
    validators: Vec<ParsedNumberValidator<T>>,
) -> Result<Vec<NumberValidator<T>>, Vec<syn::Error>> {
    // TODO: implement
    let validators: Vec<_> = validators.into_iter().map(|v| v.validator).collect();
    Ok(validators)
}

fn validate_sanitizers<T>(
    sanitizers: Vec<ParsedNumberSanitizer<T>>,
) -> Result<Vec<NumberSanitizer<T>>, Vec<syn::Error>> {
    // TODO: implement
    let sanitizers: Vec<_> = sanitizers.into_iter().map(|s| s.sanitizer).collect();
    Ok(sanitizers)
}
