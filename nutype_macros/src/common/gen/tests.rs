use proc_macro2::TokenStream;
use quote::quote;

use crate::common::models::{NumericBound, TypeName};

pub fn gen_test_should_have_consistent_lower_and_upper_boundaries<Validator>(
    type_name: &TypeName,
    validators: &Vec<Validator>,
) -> Option<TokenStream>
where
    Vec<Validator>: NumericBound,
{
    let maybe_upper = validators.upper();
    let maybe_lower = validators.lower();
    let (Some(upper), Some(lower)) = (maybe_upper, maybe_lower) else {
        return None;
    };

    let msg = format!("\nInconsistent lower and upper boundaries for type `{type_name}`\nThe upper boundary `{upper}` must be greater than or equal to the lower boundary `{lower}`\n");

    Some(quote!(
        #[test]
        fn should_have_consistent_lower_and_upper_boundaries() {
            assert!(#upper >= #lower, #msg);
        }
    ))
}
