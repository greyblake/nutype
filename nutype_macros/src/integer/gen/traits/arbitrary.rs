use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    common::models::{TypeName, Validation},
    integer::models::{IntegerGuard, IntegerInnerType, IntegerValidator},
    utils::issue_reporter::{build_github_link_with_issue, Issue},
};

pub fn gen_impl_trait_arbitrary<T: ToTokens>(
    type_name: &TypeName,
    inner_type: &IntegerInnerType,
    guard: &IntegerGuard<T>,
) -> Result<TokenStream, syn::Error> {
    let Boundary { min, max } = guard_to_boundary(inner_type, guard)?;

    let construct_value = if guard.has_validation() {
        // If by some reason we generate an invalid value, make it very easy for the user to report
        let report_issue_msg =
            build_github_link_with_issue(&Issue::ArbitraryGeneratedInvalidValue {
                inner_type: inner_type.to_string(),
            });
        let error_text =
            format!("Arbitrary generated an invalid value for {type_name}.\n\n{report_issue_msg}");
        quote!(
            Self::try_new(inner_value).expect(#error_text)
        )
    } else {
        quote!(Self::new(inner_value))
    };

    Ok(quote!(
        impl ::arbitrary::Arbitrary<'_> for #type_name {
            fn arbitrary(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<Self> {
                let inner_value: #inner_type = u.int_in_range((#min)..=(#max))?;
                Ok(#construct_value)
            }
        }

        #[inline]
        fn size_hint(_depth: usize) -> (usize, Option<usize>) {
            let n = ::core::mem::size_of::<#inner_type>();
            (n, Some(n))
        }
    ))
}

#[derive(Debug)]
struct Boundary {
    min: TokenStream,
    max: TokenStream,
}

fn guard_to_boundary<T: ToTokens>(
    inner_type: &IntegerInnerType,
    guard: &IntegerGuard<T>,
) -> Result<Boundary, syn::Error> {
    let mut boundary = Boundary {
        min: quote!(#inner_type::MIN),
        max: quote!(#inner_type::MAX),
    };

    match guard {
        IntegerGuard::WithoutValidation { sanitizers: _ } => {
            // Nothing to validate, so every possible value for the inner type is valid.
        }
        IntegerGuard::WithValidation {
            sanitizers: _,
            validation,
        } => {
            match validation {
                Validation::Custom { .. } => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "Cannot derive trait `Arbitrary` for a type with custom `with` validator.",
                    ));
                }
                Validation::Standard {
                    validators,
                    error_type_path: _,
                } => {
                    // Apply the validators to the boundaries.
                    // Since the validators were already validated, it's guaranteed that they're not
                    // contradicting each other.
                    for validator in validators {
                        match validator {
                            IntegerValidator::Greater(gt) => {
                                boundary.min = quote!(#gt + 1);
                            }
                            IntegerValidator::GreaterOrEqual(gte) => {
                                boundary.min = quote!(#gte);
                            }
                            IntegerValidator::Less(lt) => {
                                boundary.max = quote!(#lt - 1);
                            }
                            IntegerValidator::LessOrEqual(lte) => {
                                boundary.max = quote!(#lte);
                            }
                            IntegerValidator::Predicate(_) => {
                                return Err(syn::Error::new(
                                    proc_macro2::Span::call_site(),
                                    "Cannot derive trait `Arbitrary` for a type with `predicate` validator",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(boundary)
}
