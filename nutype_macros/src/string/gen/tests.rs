use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{common::models::TypeName, string::models::StringValidator};

pub fn gen_test_should_have_consistent_len_char_boundaries(
    type_name: &TypeName,
    validators: &[StringValidator],
) -> Option<TokenStream> {
    let maybe_len_char_min: Option<TokenStream> = validators
        .iter()
        .flat_map(|v| match v {
            StringValidator::LenCharMin(len) => Some(len.to_token_stream()),
            _ => None,
        })
        .next();
    let maybe_len_char_max: Option<TokenStream> = validators
        .iter()
        .flat_map(|v| match v {
            StringValidator::LenCharMax(len) => Some(len.to_token_stream()),
            _ => None,
        })
        .next();
    let (Some(len_char_min), Some(len_char_max)) = (maybe_len_char_min, maybe_len_char_max) else {
        return None;
    };

    let msg = format!("\nInconsistent lower and upper boundaries for type `{type_name}`\nThe upper boundary `{len_char_max}` must be greater than or equal to the lower boundary `{len_char_min}`\n");

    Some(quote!(
        #[test]
        fn should_have_consistent_len_char_boundaries() {
            assert!(#len_char_max >= #len_char_min, #msg);
        }
    ))
}
