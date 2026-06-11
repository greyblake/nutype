//! Shared code generation for numeric inner types (integer, float, decimal).
//!
//! Integer, float and decimal newtypes share almost all of their generated
//! code: the `__sanitize__` function, the `__validate__` function (bound checks
//! and predicate), and the validation error enum with its `Display` impl. The
//! only real differences are:
//! * float additionally supports the `Finite` validator;
//! * each kind allows a slightly different set of derivable traits (handled in
//!   the per-kind `validate.rs`, not here).
//!
//! To avoid maintaining three near-identical copies, each kind implements
//! [`NumericValidatorTokens`] and [`NumericSanitizerTokens`] for its validator
//! and sanitizer enums, and the generation below is written once against those
//! traits.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::common::{
    generate::error::gen_impl_error_trait,
    models::{ConstFn, ErrorTypePath, TypeName},
};

/// A normalized, kind-agnostic view of a single numeric validator.
///
/// Each numeric validator enum (`IntegerValidator`, `FloatValidator`,
/// `DecimalValidator`) maps onto these variants. The associated value of a
/// bound (or the predicate function) is exposed as `&dyn ToTokens`, which is all
/// the code generation needs.
pub enum NumericValidatorView<'a> {
    Greater(&'a dyn ToTokens),
    GreaterOrEqual(&'a dyn ToTokens),
    Less(&'a dyn ToTokens),
    LessOrEqual(&'a dyn ToTokens),
    Predicate(&'a dyn ToTokens),
    Finite,
}

/// Implemented by every numeric validator enum so the shared generators can
/// treat them uniformly.
pub trait NumericValidatorTokens {
    fn view(&self) -> NumericValidatorView<'_>;
}

/// Implemented by every numeric sanitizer enum. Numeric sanitizers only ever
/// carry a custom `with = ...` function (plus a `_Phantom` variant that is
/// never constructed), so a single accessor is enough.
pub trait NumericSanitizerTokens {
    /// Returns the custom sanitizer function, or `None` for the phantom variant.
    fn custom_fn(&self) -> Option<&dyn ToTokens>;
}

/// Generate the `__sanitize__` function shared by all numeric kinds.
pub fn gen_numeric_fn_sanitize<S, IT>(
    inner_type: &IT,
    sanitizers: &[S],
    const_fn: ConstFn,
) -> TokenStream
where
    S: NumericSanitizerTokens,
    IT: ToTokens,
{
    let transformations: TokenStream = sanitizers
        .iter()
        .filter_map(|san| san.custom_fn())
        .map(|custom_sanitizer| {
            quote!(
                value = (#custom_sanitizer)(value);
            )
        })
        .collect();

    quote!(
        #const_fn fn __sanitize__(mut value: #inner_type) -> #inner_type {
            #transformations
            value
        }
    )
}

/// Generate the `__validate__` function shared by all numeric kinds.
pub fn gen_numeric_fn_validate<V, IT>(
    inner_type: &IT,
    error_type_path: &ErrorTypePath,
    validators: &[V],
    const_fn: ConstFn,
) -> TokenStream
where
    V: NumericValidatorTokens,
    IT: ToTokens,
{
    let validations: TokenStream = validators
        .iter()
        .map(|validator| match validator.view() {
            NumericValidatorView::Less(exclusive_upper_bound) => {
                quote!(
                    if val >= #exclusive_upper_bound {
                        return Err(#error_type_path::LessViolated);
                    }
                )
            }
            NumericValidatorView::LessOrEqual(max) => {
                quote!(
                    if val > #max {
                        return Err(#error_type_path::LessOrEqualViolated);
                    }
                )
            }
            NumericValidatorView::Greater(exclusive_lower_bound) => {
                quote!(
                    if val <= #exclusive_lower_bound {
                        return Err(#error_type_path::GreaterViolated);
                    }
                )
            }
            NumericValidatorView::GreaterOrEqual(min) => {
                quote!(
                    if val < #min {
                        return Err(#error_type_path::GreaterOrEqualViolated);
                    }
                )
            }
            NumericValidatorView::Predicate(custom_is_valid_fn) => {
                quote!(
                    if !(#custom_is_valid_fn)(&val) {
                        return Err(#error_type_path::PredicateViolated);
                    }
                )
            }
            NumericValidatorView::Finite => {
                quote!(
                    if !val.is_finite() {
                        return Err(#error_type_path::FiniteViolated);
                    }
                )
            }
        })
        .collect();

    quote!(
        #const_fn fn __validate__(val: &#inner_type) -> ::core::result::Result<(), #error_type_path> {
            let val = *val;
            #validations
            Ok(())
        }
    )
}

/// Generate the validation error enum (definition + `Display` + `Error`) shared
/// by all numeric kinds.
pub fn gen_numeric_validation_error_type<V>(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[V],
) -> TokenStream
where
    V: NumericValidatorTokens,
{
    let definition = gen_definition(error_type_path, validators);
    let impl_display_trait = gen_impl_display_trait(type_name, error_type_path, validators);
    let impl_error_trait = gen_impl_error_trait(error_type_path);

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #definition

        #impl_display_trait
        #impl_error_trait
    }
}

fn gen_definition<V>(error_type_path: &ErrorTypePath, validators: &[V]) -> TokenStream
where
    V: NumericValidatorTokens,
{
    let error_variants: TokenStream = validators
        .iter()
        .map(|validator| match validator.view() {
            NumericValidatorView::Greater(_) => quote!(GreaterViolated,),
            NumericValidatorView::GreaterOrEqual(_) => quote!(GreaterOrEqualViolated,),
            NumericValidatorView::Less(_) => quote!(LessViolated,),
            NumericValidatorView::LessOrEqual(_) => quote!(LessOrEqualViolated,),
            NumericValidatorView::Predicate(_) => quote!(PredicateViolated,),
            NumericValidatorView::Finite => quote!(FiniteViolated,),
        })
        .collect();

    quote! {
        #[allow(clippy::enum_variant_names)]
        pub enum #error_type_path {
            #error_variants
        }
    }
}

fn gen_impl_display_trait<V>(
    type_name: &TypeName,
    error_type_path: &ErrorTypePath,
    validators: &[V],
) -> TokenStream
where
    V: NumericValidatorTokens,
{
    let match_arms = validators.iter().map(|validator| match validator.view() {
        NumericValidatorView::Greater(val) => quote! {
             #error_type_path::GreaterViolated => write!(f, "{} is too small. The value must be greater than {:#?}.", stringify!(#type_name), #val)
        },
        NumericValidatorView::GreaterOrEqual(val) => quote! {
             #error_type_path::GreaterOrEqualViolated => write!(f, "{} is too small. The value must be greater or equal to {:#?}.", stringify!(#type_name), #val)
        },
        NumericValidatorView::Less(val) => quote! {
             #error_type_path::LessViolated => write!(f, "{} is too big. The value must be less than {:#?}.", stringify!(#type_name), #val)
        },
        NumericValidatorView::LessOrEqual(val) => quote! {
             #error_type_path::LessOrEqualViolated => write!(f, "{} is too big. The value must be less or equal to {:#?}.", stringify!(#type_name), #val)
        },
        NumericValidatorView::Predicate(_) => quote! {
             #error_type_path::PredicateViolated => write!(f, "{} failed the predicate test.", stringify!(#type_name))
        },
        NumericValidatorView::Finite => quote! {
             #error_type_path::FiniteViolated => write!(f, "{} is not finite.", stringify!(#type_name))
        },
    });

    quote! {
        impl ::core::fmt::Display for #error_type_path {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    }
}
