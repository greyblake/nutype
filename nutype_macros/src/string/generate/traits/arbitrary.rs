use kinded::Kinded;
use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{
    common::models::{TypeName, Validation, ValueOrExpr},
    string::models::{StringGuard, StringSanitizer, StringValidator},
    utils::issue_reporter::{Issue, build_github_link_with_issue},
};

pub fn gen_impl_trait_arbitrary(
    type_name: &TypeName,
    guard: &StringGuard,
) -> Result<TokenStream, syn::Error> {
    let construct_value = if guard.has_validation() {
        // If by some reason we generate an invalid value, make it very easy for the user to report
        let report_issue_msg =
            build_github_link_with_issue(&Issue::ArbitraryGeneratedInvalidValue {
                inner_type: "String".to_string(),
            });
        let type_name = type_name.to_string();
        quote!(
            Self::try_new(inner_value.clone()).unwrap_or_else(|err| {
                // Panic with the maximum details about what went wrong
                panic!("\nArbitrary generated an invalid value for {}.\nInvalid inner value: {:?}\nValidation error: {:?}\n\n{}", #type_name, inner_value, err, #report_issue_msg);
            })
        )
    } else {
        quote!(Self::new(inner_value))
    };

    let maybe_spec = build_specification(guard)?;
    let generate_inner_value = gen_generate_valid_inner_value(&maybe_spec);
    let size_hint = gen_size_hint(&maybe_spec);

    Ok(quote!(
        impl ::arbitrary::Arbitrary<'_> for #type_name {
            fn arbitrary(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<Self> {
                let inner_value: String = {
                    #generate_inner_value
                };
                Ok(#construct_value)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                #size_hint
            }
        }
    ))
}

fn gen_size_hint(maybe_spec: &Option<Specification>) -> TokenStream {
    match maybe_spec {
        Some(spec) => {
            let Specification {
                min_len, max_len, ..
            } = spec;
            quote!(
                let min_size_hint = #min_len * core::mem::size_of::<char>();
                let max_size_hint = #max_len * core::mem::size_of::<char>();
                (min_size_hint, Some(max_size_hint))
            )
        }
        None => {
            // That corresponds to `quote!(u.arbitrary()?)` implementation
            quote!(<String as ::arbitrary::Arbitrary<'_>>::size_hint(depth))
        }
    }
}

fn gen_generate_valid_inner_value(maybe_spec: &Option<Specification>) -> TokenStream {
    match maybe_spec {
        Some(spec) => gen_generate_valid_inner_value_with_validators(spec),
        None => {
            // When there is no validation, then we can just simply delegate to the arbitrary
            // crate, and the job is done.
            quote!(u.arbitrary()?)
        }
    }
}

/// Subset of StringSanitizer, which is is possible to handle and is relevant for generating
/// arbitrary strings.
#[derive(Kinded)]
enum RelevantSanitizer {
    Trim,
}

/// Subset of StringValidator, which is is possible to handle and is relevant for generating
/// arbitrary strings.
#[derive(Kinded)]
enum RelevantValidator {
    LenCharMin(ValueOrExpr<usize>),
    LenCharMax(ValueOrExpr<usize>),
}

/// Final specification to generate an arbitrary valid string
struct Specification {
    has_trim: bool,
    min_len: ValueOrExpr<usize>,
    max_len: ValueOrExpr<usize>,
}

/// If max length is not specified, then sum of min_len + this offset will be used.
const DEFAULT_LEN_OFFSET: usize = 16;

fn build_specification(guard: &StringGuard) -> Result<Option<Specification>, syn::Error> {
    match guard {
        StringGuard::WithoutValidation { .. } => Ok(None),
        StringGuard::WithValidation {
            sanitizers,
            validation,
        } => {
            let validators = get_validators(validation)?;
            let relevant_validators = filter_validators(validators)?;
            let relevant_sanitizers = filter_sanitizers(sanitizers)?;

            let has_trim = relevant_sanitizers
                .iter()
                .any(|s| matches!(s, RelevantSanitizer::Trim));
            let min_len = relevant_validators
                .iter()
                .find_map(|v| {
                    if let RelevantValidator::LenCharMin(value) = v {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| ValueOrExpr::Value(0));
            let max_len = relevant_validators
                .iter()
                .find_map(|v| {
                    if let RelevantValidator::LenCharMax(value) = v {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| min_len.clone() + DEFAULT_LEN_OFFSET);

            let spec = Specification {
                has_trim,
                min_len,
                max_len,
            };
            Ok(Some(spec))
        }
    }
}

fn get_validators(
    validation: &Validation<StringValidator>,
) -> Result<&[StringValidator], syn::Error> {
    match validation {
        Validation::Standard { validators, .. } => Ok(validators),
        Validation::Custom { .. } => {
            let msg = "It's not possible to derive `Arbitrary` trait for a type with custom validation.\nYou have to implement `Arbitrary` trait on you own.";
            Err(syn::Error::new(Span::call_site(), msg))
        }
    }
}

fn filter_validators(validators: &[StringValidator]) -> Result<Vec<RelevantValidator>, syn::Error> {
    validators.iter().map(|v| {
        match v {
            StringValidator::LenCharMin(value) => Ok(RelevantValidator::LenCharMin(value.clone())),
            StringValidator::LenCharMax(value) => Ok(RelevantValidator::LenCharMax(value.clone())),
            // In context of generating an arbitrary string NotEmpty is the same as LenCharMin(1)
            StringValidator::NotEmpty => Ok(RelevantValidator::LenCharMin(ValueOrExpr::Value(1))),
            StringValidator::Predicate(_) => {
                let msg = "It's not possible to derive `Arbitrary` trait for a type with `predicate` validator.\nYou have to implement `Arbitrary` trait on you own.";
                Err(syn::Error::new(Span::call_site(), msg))
            }
            StringValidator::Regex(_) => {
                let msg = "It's not possible to derive `Arbitrary` trait for a type with `regex` validator.\nYou have to implement `Arbitrary` trait on you own.";
                Err(syn::Error::new(Span::call_site(), msg))
            }
        }
    }).collect()
}

fn filter_sanitizers(sanitizers: &[StringSanitizer]) -> Result<Vec<RelevantSanitizer>, syn::Error> {
    sanitizers.iter().filter_map(|s| {
        match s {
            // Trim is relevant, because trimming a space can decrease string length and cause
            // violation of len_char_min validation.
            StringSanitizer::Trim => Some(Ok(RelevantSanitizer::Trim)),
            // lowercase and uppercase sanitizers do not overlap with any of the validation rules,
            // so we can ignore them
            StringSanitizer::Lowercase => None,
            StringSanitizer::Uppercase => None,
            StringSanitizer::With(_) => {
                let msg = "It's not possible to derive `Arbitrary` trait for a type with `with` sanitizer.\nYou have to implement `Arbitrary` trait on you own.";
                Some(Err(syn::Error::new(Span::call_site(), msg)))
            }
        }
    }).collect()
}

fn gen_generate_valid_inner_value_with_validators(spec: &Specification) -> TokenStream {
    let Specification {
        has_trim,
        min_len,
        max_len,
    } = spec;

    if *has_trim {
        quote!(
            // Pick randomly a target length
            let target_len = u.int_in_range((#min_len)..=(#max_len))?;
            // Generate string `output` that matches the target_len
            let mut output = String::with_capacity(target_len * 2);
            for _ in 0..target_len {
                let ch: char = u.arbitrary()?;
                output.push(ch);
            }
            // Make sure that the generated string matches the target_len
            // after trimming the spaces.
            loop {
                let count = output.trim().chars().count();

                match count.cmp(&target_len) {
                    core::cmp::Ordering::Equal => {
                        break;
                    }
                    core::cmp::Ordering::Less => {
                        // Try luck one more time: trim the spaces and add another char.
                        // NOTE: This is inefficient, but it's not expected to happen often.
                        output = output.trim().to_string();
                        let new_char: char = u.arbitrary()?;
                        output.push(new_char);
                    }
                    core::cmp::Ordering::Greater => {
                        unreachable!(
                            "This should never happened. Generated string is longer then target_len."
                        );
                    }
                }
            }
            // Return the output string
            output
        )
    } else {
        quote!(
            // Pick randomly a target length
            let target_len = u.int_in_range((#min_len)..=(#max_len))?;
            // Generate string `output` that matches the target_len
            let mut output = String::with_capacity(target_len * 2);
            for _ in 0..target_len {
                let ch: char = u.arbitrary()?;
                output.push(ch);
            }
            // Return the output string
            output
        )
    }
}
