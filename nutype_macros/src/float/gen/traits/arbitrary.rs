use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    common::models::TypeName,
    float::models::{
        FloatGuard, FloatInnerType, FloatSanitizer, FloatSanitizerKind, FloatValidator,
        FloatValidatorKind,
    },
    utils::issue_reporter::{build_github_link_with_issue, Issue},
};

pub fn gen_impl_trait_arbitrary<T: ToTokens>(
    type_name: &TypeName,
    inner_type: &FloatInnerType,
    guard: &FloatGuard<T>,
) -> Result<TokenStream, syn::Error> {
    let construct_value = if guard.has_validation() {
        // If by some reason we generate an invalid value, make it very easy for the user to report
        let report_issue_msg =
            build_github_link_with_issue(&Issue::ArbitraryGeneratedInvalidValue {
                inner_type: inner_type.to_string(),
            });
        let type_name = type_name.to_string();
        quote!(
            Self::new(inner_value).unwrap_or_else(|err| {
                // Panic with the maximum details about what went wrong
                panic!("\nArbitrary generated an invalid value for {}.\nInvalid inner value: {:?}\nValidation error: {:?}\n\n{}", #type_name, inner_value, err, #report_issue_msg);
            })
        )
    } else {
        quote!(Self::new(inner_value))
    };

    let generate_inner_value = gen_generate_valid_inner_value(inner_type, guard)?;

    Ok(quote!(
        impl ::arbitrary::Arbitrary<'_> for #type_name {
            fn arbitrary(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<Self> {
                let inner_value: #inner_type = {
                    #generate_inner_value
                };
                Ok(#construct_value)
            }
        }
    ))
}

/// Generates a code that generates a valid inner value.
fn gen_generate_valid_inner_value<T: ToTokens>(
    inner_type: &FloatInnerType,
    guard: &FloatGuard<T>,
) -> Result<TokenStream, syn::Error> {
    match guard {
        FloatGuard::WithoutValidation { .. } => {
            // When there is no validation, then we can just simply delegate to the arbitrary
            // crate, and the job is done.
            Ok(quote!(u.arbitrary()?))
        }
        FloatGuard::WithValidation {
            sanitizers,
            validators,
        } => {
            // When there is validation, then we need to generate a valid value.
            gen_generate_valid_inner_value_with_validators(inner_type, sanitizers, validators)
        }
    }
}

fn gen_generate_valid_inner_value_with_validators<T: ToTokens>(
    inner_type: &FloatInnerType,
    sanitizers: &[FloatSanitizer<T>],
    validators: &[FloatValidator<T>],
) -> Result<TokenStream, syn::Error> {
    let validator_kinds: Vec<FloatValidatorKind> = validators.iter().map(|v| v.kind()).collect();
    let sanitizer_kinds: Vec<FloatSanitizerKind> = sanitizers.iter().map(|s| s.kind()).collect();

    if validator_kinds.contains(&FloatValidatorKind::Predicate) {
        let span = Span::call_site();
        let msg = "It's not possible to derive `Arbitrary` trait for a type with `predicate` validator.\nYou have to implement `Arbitrary` trait on you own.";
        return Err(syn::Error::new(span, msg));
    }
    if sanitizer_kinds.contains(&FloatSanitizerKind::With) {
        let span = Span::call_site();
        let msg = "It's not possible to derive `Arbitrary` trait for a type with `with` sanitizer and validations.\nYou have to implement `Arbitrary` trait on you own.";
        return Err(syn::Error::new(span, msg));
    }

    let basic_value_kind = compute_basic_value_kind(&validator_kinds);
    let basic_value = generate_basic_value(inner_type, basic_value_kind);
    let boundaries = compute_boundaries(validators);

    Ok(normalize_basic_value_for_boundaries(
        inner_type,
        basic_value,
        boundaries,
    ))
}

fn normalize_basic_value_for_boundaries(
    inner_type: &FloatInnerType,
    basic_value: TokenStream,
    boundaries: Boundaries,
) -> TokenStream {
    match (boundaries.lower, boundaries.upper) {
        (Some(lower), Some(upper)) => {
            // In this case we don't use `basic_value` we generate a new value that lays in between
            // 0.0 and 1.0 and then scale it to the range of the boundaries.
            let arbitrary_in_01_range = gen_in_01_range(inner_type);

            let lower_value = &lower.value;
            let upper_value = &upper.value;
            let adjust_x_lower = gen_adjust_x_for_lower_boundary(inner_type, &lower);
            let adjust_x_upper = gen_adjust_x_for_upper_boundary(inner_type, &lower);
            quote! {
                let from0to1 = #arbitrary_in_01_range;

                // Scale range [0; 1] to the range of the boundaries
                let range = (#upper_value - #lower_value).abs();
                let x = #lower_value + from0to1 * range;

                // Make sure we satisfy the exclusive boundaries
                let x = #adjust_x_lower;
                let x = #adjust_x_upper;
                x
            }
        }
        (Some(lower), None) => {
            let lower_value = &lower.value;
            let adjust_x = gen_adjust_x_for_lower_boundary(inner_type, &lower);
            quote! {
                // Compute initial basic value
                let basic_value = #basic_value;
                let positive_basic_value = basic_value.abs();
                let x = positive_basic_value + #lower_value;
                #adjust_x
            }
        }
        (None, Some(upper)) => {
            let upper_value = &upper.value;
            let adjust_x = gen_adjust_x_for_upper_boundary(inner_type, &upper);
            quote! {
                // Compute initial basic value
                let basic_value = #basic_value;
                let negative_basic_value = -basic_value.abs();
                let x = negative_basic_value + #upper_value;
                #adjust_x
            }
        }
        (None, None) => basic_value,
    }
}

fn gen_adjust_x_for_upper_boundary(
    float_type: &FloatInnerType,
    upper_boundary: &Boundary,
) -> TokenStream {
    if upper_boundary.is_inclusive {
        quote! { x }
    } else {
        let upper_value = &upper_boundary.value;
        let correction_delta = correction_delta_for_float_type(float_type);
        quote! {
            if x >= #upper_value {
                x - #correction_delta
            } else {
                x
            }
        }
    }
}

fn gen_adjust_x_for_lower_boundary(
    float_type: &FloatInnerType,
    lower_boundary: &Boundary,
) -> TokenStream {
    if lower_boundary.is_inclusive {
        quote! { x }
    } else {
        let lower_value = &lower_boundary.value;
        let correction_delta = correction_delta_for_float_type(float_type);
        quote! {
            if x <= #lower_value {
                // Since there is no upper boundary, we are free to add any positive value here
                // to adjust so we can satisfy the exclusive lower boundary.
                x + #correction_delta
            } else {
                x
            }
        }
    }
}

/// A tiny value that is used to correct the value to satisfy the exclusive boundaries if
/// necessary.
/// For example, if the constraint is `greater = 0.0`, then and we obtain exactly `0.0` when
/// generating a pseudo-random value, then we need to add a tiny value to it to make it
/// satisfy `x > 0.0` check.
///
/// Unfortunately things like `f32::EPSILON` or `f64::EPSILON` are not suitable for this purpose.
/// The constants are found experimentally.
fn correction_delta_for_float_type(float_type: &FloatInnerType) -> TokenStream {
    match float_type {
        FloatInnerType::F32 => quote!(0.000_002),
        FloatInnerType::F64 => quote!(0.000_000_000_000_004),
    }
}

/// Generate a code snippet that generates a random float in range [0; 1].
/// Assumptions
/// * There is variable `u` in the given context which is a value of `arbitrary::Unstructured`.
fn gen_in_01_range(float_type: &FloatInnerType) -> TokenStream {
    let int_type = match float_type {
        FloatInnerType::F32 => quote!(u32),
        FloatInnerType::F64 => quote!(u64),
    };

    quote! (
        {                                                                                // {
            let random_int: #int_type = u.arbitrary()?;                                  //     let random_int: u32 = u.arbitrary()?;
            (random_int as #float_type / #int_type::MAX as #float_type) as #float_type   //     (random_int as f32 / u32::MAX as f32) as f32
        }                                                                                // }
    )
}

struct Boundaries {
    lower: Option<Boundary>,
    upper: Option<Boundary>,
}

struct Boundary {
    value: TokenStream,
    is_inclusive: bool,
}

/// Describes a type of initial basic value that has to be generated.
enum BasicValueKind {
    /// All float values including NaN and Infinity.
    All,

    /// All float values except NaN
    NotNaN,

    /// All float values except NaN and infinities
    Finite,
}

fn compute_basic_value_kind(validators: &[FloatValidatorKind]) -> BasicValueKind {
    let has_boundaries = || {
        validators.contains(&FloatValidatorKind::Greater)
            || validators.contains(&FloatValidatorKind::GreaterOrEqual)
            || validators.contains(&FloatValidatorKind::Less)
            || validators.contains(&FloatValidatorKind::LessOrEqual)
    };

    if validators.contains(&FloatValidatorKind::Finite) {
        BasicValueKind::Finite
    } else if has_boundaries() {
        BasicValueKind::NotNaN
    } else {
        BasicValueKind::All
    }
}

fn compute_boundaries<T: ToTokens>(validators: &[FloatValidator<T>]) -> Boundaries {
    let mut lower = None;
    let mut upper = None;

    // NOTE: It's guaranteed that either Greater or GreaterOrEqual present, but not both,
    // Same for Less and LessOrEqual.
    // This handled by prior validation.
    for validator in validators {
        match validator {
            FloatValidator::Greater(expr) => {
                let value = quote!(#expr);
                let is_inclusive = false;
                lower = Some(Boundary {
                    value,
                    is_inclusive,
                });
            }
            FloatValidator::GreaterOrEqual(expr) => {
                let value = quote!(#expr);
                let is_inclusive = true;
                lower = Some(Boundary {
                    value,
                    is_inclusive,
                });
            }
            FloatValidator::Less(expr) => {
                let value = quote!(#expr);
                let is_inclusive = false;
                upper = Some(Boundary {
                    value,
                    is_inclusive,
                });
            }
            FloatValidator::LessOrEqual(expr) => {
                let value = quote!(#expr);
                let is_inclusive = true;
                upper = Some(Boundary {
                    value,
                    is_inclusive,
                });
            }
            FloatValidator::Finite | FloatValidator::Predicate(..) => {
                // We don't care about these validators here.
            }
        }
    }

    Boundaries { lower, upper }
}

fn generate_basic_value(inner_type: &FloatInnerType, kind: BasicValueKind) -> TokenStream {
    match kind {
        BasicValueKind::All => quote!(u.arbitrary()?),
        BasicValueKind::NotNaN => generate_not_nan_float(inner_type),
        BasicValueKind::Finite => generate_finite_float(inner_type),
    }
}

// Generate a code that generates a float which is not infinite and not NaN.
fn generate_finite_float(inner_type: &FloatInnerType) -> TokenStream {
    let condition = quote!(value.is_finite());
    generate_float_with_condition(inner_type, condition)
}

// Generate a code that generates a float which is not NaN
fn generate_not_nan_float(inner_type: &FloatInnerType) -> TokenStream {
    let condition = quote!(!value.is_nan());
    generate_float_with_condition(inner_type, condition)
}

// Generates a block of code that uses arbitrary and deterministic mutations to find a value that
// matches the given condition.
// IMPORTANT: The condition must be something like check against NaN or infinity and should not be
// a check against a range of values (otherwise it may loop forever).
//
// The generated code takes the following assumptions:
// * There is variable `u` in the given context which is a value of `arbitrary::Unstructured`.
// * `condition` is a closure that does a check against variable `value` and returns a bool.
fn generate_float_with_condition(
    inner_type: &FloatInnerType,
    condition: TokenStream,
) -> TokenStream {
    quote! (
        {
            let condition = |value: #inner_type| #condition;

            'outer: loop {
                let original_value: #inner_type = u.arbitrary()?;

                if condition(original_value) {
                    break original_value;
                } else {
                    // If the original value obtained from arbitrary does not match the condition,
                    // we try to mangle/randomize it deterministically in a loop 100 times, until we
                    // reach out for another value from arbitrary.
                    // Generally it must be more than enough, cause what we typically need is to avoid
                    // NaN and infinity.

                    // This returns
                    // * [u8; 4] for f32
                    // * [u8; 8] for f64
                    let mut bytes = original_value.to_be_bytes();
                    for i in 0..100 {
                        // With every iteration we modify next single byte by adding `i` value to
                        // it.
                        let index = i % std::mem::size_of::<#inner_type>();
                        bytes[index] = bytes[index].wrapping_add((i % 256) as u8);

                        // Try to convert the bytes back to float in both BE and NE formats and see
                        // if we get something what we need
                        let new_float_be = #inner_type::from_be_bytes(bytes);
                        if condition(new_float_be) {
                            break 'outer new_float_be;
                        }
                        let new_float_ne = #inner_type::from_ne_bytes(bytes);
                        if condition(new_float_ne) {
                            break 'outer new_float_ne;
                        }
                    }
                }
            }
        }
    )
}
