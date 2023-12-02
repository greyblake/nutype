use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    common::models::TypeName,
    integer::models::{IntegerGuard, IntegerInnerType, IntegerValidator},
};

pub fn gen_impl_trait_arbitrary<T: ToTokens>(
    type_name: &TypeName,
    inner_type: &IntegerInnerType,
    guard: &IntegerGuard<T>,
) -> TokenStream {
    let Boundary { min, max } = guard_to_boundary(inner_type, guard);

    let construct_value = if guard.has_validation() {
        // TODO:
        // * Use a constant instead of hardcoded URL
        // * Add predefined `title` and `body` parameters so there is already some text in the issue
        // * Move it all into some helper function that can be reused
        let error_text = format!("Arbitrary generated an invalid value for {type_name}.\nPlease report the issue at https://github.com/greyblake/nutype/issues/new");
        quote!(
            Self::new(inner_value).expect(#error_text)
        )
    } else {
        quote!(Self::new(inner_value))
    };

    quote!(
        impl ::arbitrary::Arbitrary<'_> for #type_name {
            fn arbitrary(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<Self> {
                let inner_value: #inner_type = u.int_in_range((#min)..=(#max))?;
                Ok(#construct_value)
            }
        }
    )
}

#[derive(Debug)]
struct Boundary {
    min: TokenStream,
    max: TokenStream,
}

fn guard_to_boundary<T: ToTokens>(
    inner_type: &IntegerInnerType,
    guard: &IntegerGuard<T>,
) -> Boundary {
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
            validators,
        } => {
            // Apply validators to the boundaries.
            // Since validators were already were validated, it's guaranteed that they're not
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
                        // TODO: turn into an error
                        panic!("Cannot derive Arbitrary for a type with a predicate validator");
                    }
                }
            }
        }
    }

    boundary
}
