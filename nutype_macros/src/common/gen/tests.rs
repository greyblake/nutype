use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Generics;

use crate::common::models::{ConstAssign, NumericBound, TypeName};

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

    let msg = format!(
        "
Inconsistent lower and upper boundaries for type `{type_name}`
The upper boundary `{upper}` must be greater than or equal to the lower boundary `{lower}`
Note: the test is generated automatically by #[nutype] macro.
"
    );

    Some(quote!(
        #[test]
        fn should_have_consistent_lower_and_upper_boundaries() {
            assert!(#upper >= #lower, #msg);
        }
    ))
}

pub fn gen_test_should_have_valid_default_value(
    type_name: &TypeName,
    generics: &Generics,
    maybe_default_value: &Option<syn::Expr>,
    has_validation: bool,
) -> Option<TokenStream> {
    if !has_validation {
        // If there is no validation, then every possible default value will be valid,
        // so there is no need to generate the test.
        return None;
    }

    if !generics.params.is_empty() {
        // If the type has generics, then it is not possible to generate the test,
        // because the test would require concrete types to instantiate default value.
        //
        // It could be tempting to generate a test using some concrete types (e.g. `i32` for `T`),
        // but it's not possible to guarantee that the type we pick will match the boundaries on `T`.
        return None;
    }

    let default_value: TokenStream = maybe_default_value.as_ref()?.to_token_stream();

    let msg = format!(
        "
Type `{type_name}` has invalid default value `{default_value}`
Note: the test is generated automatically by #[nutype] macro
"
    );

    Some(quote!(
        #[test]
        fn should_have_valid_default_value() {
            let default_inner_value = #type_name::default().into_inner();
            // Typically `::default()` would already panic, but in case if the panic is removed by
            // some reason we still want the test to fail.
            #type_name::try_new(default_inner_value).expect(#msg);
        }
    ))
}

pub fn gen_associated_consts_should_be_valid(
    type_name: &TypeName,
    associated_consts: &[ConstAssign],
) -> TokenStream {
    // quote! {}
    let tests_consts = associated_consts.iter().map(
        |ConstAssign {
             const_name,
             const_value,
             ..
         }| {
            let err_msg = format!(
                "
Type `{type_name}` has invalid associated const `{const_name}` of value `{const_value}`.
Note: the test is generated automatically by #[nutype] macro
"
            );

            let test_name = format_ident!("associated_const_{const_name}_should_have_valid_value");
            quote! {
                #[test]
                fn  #test_name () {
                    let inner_value = #type_name::#const_name.into_inner();

                    #type_name::try_new(inner_value).expect(#err_msg);
                }
            }
        },
    );

    quote! { #(#tests_consts)* }
}
