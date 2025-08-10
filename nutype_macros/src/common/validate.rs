use kinded::Kinded;
use proc_macro2::Span;

use super::{
    r#generate::error::gen_error_type_name,
    models::{
        DeriveTrait, Guard, NumericBoundValidator, RawGuard, SpannedDeriveTrait, SpannedItem,
        TypeName, Validation,
    },
    parse::RawValidation,
};

/// Some kind of template method to validate a guard of all types.
pub fn validate_guard<RawSanitizer, RawValidator, Sanitizer, Validator>(
    raw_guard: RawGuard<RawSanitizer, RawValidator>,
    type_name: &TypeName,
    validate_validators: impl FnOnce(Vec<RawValidator>) -> Result<Vec<Validator>, syn::Error>,
    validate_sanitizers: impl FnOnce(Vec<RawSanitizer>) -> Result<Vec<Sanitizer>, syn::Error>,
) -> Result<Guard<Sanitizer, Validator>, syn::Error> {
    let RawGuard {
        sanitizers: raw_sanitizers,
        validation: maybe_raw_validation,
    } = raw_guard;

    let sanitizers = validate_sanitizers(raw_sanitizers)?;

    let Some(raw_validation) = maybe_raw_validation else {
        return Ok(Guard::WithoutValidation { sanitizers });
    };

    let validation = match raw_validation {
        RawValidation::Standard { validators } => {
            let error_type_path = gen_error_type_name(type_name);
            let validators = validate_validators(validators)?;
            Validation::Standard {
                validators,
                error_type_path,
            }
        }
        RawValidation::Custom { with, error } => {
            let error_type_path = error;
            Validation::Custom {
                with,
                error_type_path,
            }
        }
    };
    Ok(Guard::WithValidation {
        sanitizers,
        validation,
    })
}

pub fn validate_duplicates<T>(
    items: &[SpannedItem<T>],
    build_error_msg: impl Fn(<T as Kinded>::Kind) -> String,
) -> Result<(), syn::Error>
where
    T: Kinded,
{
    if let Some((item1, item2)) = detect_items_of_same_kind(items) {
        assert_eq!(item1.kind(), item2.kind());
        let kind = item1.kind();
        let msg = build_error_msg(kind);
        let span = join_spans_or_last(item1.span(), item2.span());
        let err = syn::Error::new(span, msg);
        return Err(err);
    }
    Ok(())
}

fn detect_items_of_same_kind<T: Kinded>(items: &[T]) -> Option<(&T, &T)> {
    // Note: this has O(n^2) complexity, but it's not a problem, because size of collection is < 10.
    for (i1, item1) in items.iter().enumerate() {
        for (i2, item2) in items.iter().enumerate() {
            if i1 != i2 && item1.kind() == item2.kind() {
                return Some((item1, item2));
            }
        }
    }
    None
}

fn join_spans_or_last(span1: Span, span2: Span) -> Span {
    span1.join(span2).unwrap_or(span2)
}

macro_rules! find_bound_variant {
    ($validators:ident, $method:ident) => {
        $validators
            .iter()
            .flat_map(|validator| {
                if let Some(value) = validator.item.$method() {
                    Some(SpannedItem::new(value, validator.span()))
                } else {
                    None
                }
            })
            .next()
    };
}

pub fn validate_numeric_bounds<V, T>(validators: &[SpannedItem<V>]) -> Result<(), syn::Error>
where
    V: NumericBoundValidator<T>,
    T: Clone + PartialOrd,
{
    let maybe_greater = find_bound_variant!(validators, greater);
    let maybe_greater_or_equal = find_bound_variant!(validators, greater_or_equal);
    let maybe_less = find_bound_variant!(validators, less);
    let maybe_less_or_equal = find_bound_variant!(validators, less_or_equal);

    // greater VS greater_or_equal
    //
    if let (Some(_), Some(ge)) = (maybe_greater.clone(), maybe_greater_or_equal.clone()) {
        let msg = "The lower bound can be specified with EITHER `greater` OR `greater_or_equal`, but not both.";
        let err = syn::Error::new(ge.span(), msg);
        return Err(err);
    }
    // less VS less_or_equal
    //
    if let (Some(_), Some(le)) = (maybe_less.clone(), maybe_less_or_equal.clone()) {
        let msg =
            "The upper bound can be specified with EITHER `less` OR `less_or_equal`, but not both.";
        let err = syn::Error::new(le.span(), msg);
        return Err(err);
    }

    // less VS greater
    if let (Some(lower), Some(upper)) = (maybe_greater.clone(), maybe_less.clone())
        && lower.item >= upper.item {
            let msg = "The lower bound (`greater`) cannot be equal or higher than the upper bound (`less`).";
            let err = syn::Error::new(upper.span(), msg);
            return Err(err);
        }

    let maybe_lower_bound = maybe_greater.or(maybe_greater_or_equal);
    let maybe_upper_bound = maybe_less.or(maybe_less_or_equal);

    // less_or_equal VS greater_or_equal
    //
    if let (Some(lower), Some(upper)) = (maybe_lower_bound, maybe_upper_bound)
        && lower.item > upper.item {
            let msg = "The lower bound (`greater` or `greater_or_equal`) cannot be greater than the upper bound (`less or `less_or_equal`).\nSometimes we all need a little break.";
            let err = syn::Error::new(upper.span(), msg);
            return Err(err);
        }

    Ok(())
}

pub fn validate_traits_from_xor_try_from(
    spanned_derive_traits: &[SpannedDeriveTrait],
) -> Result<(), syn::Error> {
    let maybe_from = spanned_derive_traits
        .iter()
        .find(|dt| dt.item == DeriveTrait::From);
    let maybe_try_from = spanned_derive_traits
        .iter()
        .find(|dt| dt.item == DeriveTrait::TryFrom);

    match (maybe_from, maybe_try_from) {
        (Some(_from), Some(try_from)) => {
            let msg = "There is no need to derive `TryFrom` when `From` is already derived.\nThere is a blanket implementation for `TryFrom` in std.";
            let err = syn::Error::new(try_from.span(), msg);
            Err(err)
        }
        _ => Ok(()),
    }
}
