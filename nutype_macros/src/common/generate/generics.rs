//! Utilities for handling generics and where clauses in code generation.
//!
//! This module provides helper functions that properly handle `where` clauses
//! when generating impl blocks, including support for Higher-Ranked Trait Bounds (HRTB).

use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

/// Split generics for use in impl blocks.
///
/// This properly separates the generics into three parts:
/// - `impl_generics`: Goes after `impl` keyword (e.g., `<T: Clone>`)
/// - `type_generics`: Goes after type name (e.g., `<T>`)
/// - `where_clause`: Goes at the end of impl signature (e.g., `where T: Default`)
///
/// # Example
///
/// For `struct Foo<T: Clone>(T) where T: Default`:
///
/// ```ignore
/// let split = split_generics_for_impl(&generics);
/// quote! {
///     impl #impl_generics SomeTrait for Foo #type_generics #where_clause {
///         // ...
///     }
/// }
/// ```
///
/// Generates:
/// ```ignore
/// impl<T: Clone> SomeTrait for Foo<T> where T: Default {
///     // ...
/// }
/// ```
pub struct SplitGenerics {
    pub impl_generics: TokenStream,
    pub type_generics: TokenStream,
    pub where_clause: TokenStream,
}

impl SplitGenerics {
    pub fn new(generics: &Generics) -> Self {
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        Self {
            impl_generics: quote!(#impl_generics),
            type_generics: quote!(#type_generics),
            where_clause: quote!(#where_clause),
        }
    }
}

/// Add a bound to all type parameters in generics.
///
/// This adds the bound to inline type parameters.
///
/// # Arguments
/// * `generics` - The original generics
/// * `bound` - The bound to add (e.g., `Display`, `Serialize`)
///
/// # Example
///
/// Input: `<T, U>` with bound `Display`
/// Output: `<T: Display, U: Display>`
pub fn add_bound_to_all_type_params(generics: &Generics, bound: syn::TypeParamBound) -> Generics {
    let mut result = generics.clone();
    for param in &mut result.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param.bounds.push(bound.clone());
        }
    }
    result
}

/// Add a generic parameter (typically a lifetime) to generics.
///
/// The parameter is added at the end of the params list.
///
/// # Example
///
/// Input: `<T, U>` with param `'de`
/// Output: `<T, U, 'de>`
pub fn add_generic_param(generics: &Generics, param: syn::GenericParam) -> Generics {
    let mut result = generics.clone();
    result.params.push(param);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_split_generics_simple() {
        let generics: Generics = parse_quote!(<T: Clone>);
        let split = SplitGenerics::new(&generics);

        // Just verify it doesn't panic and produces some output
        assert!(!split.impl_generics.is_empty());
        assert!(!split.type_generics.is_empty());
    }

    #[test]
    fn test_split_generics_with_where_clause() {
        // Parse a full struct to get generics with where clause
        let item: syn::ItemStruct = parse_quote! {
            struct Foo<T> where T: Clone { field: T }
        };
        let split = SplitGenerics::new(&item.generics);

        // Verify where clause is captured
        assert!(!split.where_clause.is_empty());
    }

    #[test]
    fn test_split_generics_with_hrtb() {
        // Parse a full struct to get generics with HRTB where clause
        let item: syn::ItemStruct = parse_quote! {
            struct Foo<C> where for<'a> &'a C: IntoIterator { field: C }
        };
        let split = SplitGenerics::new(&item.generics);

        // Verify HRTB where clause is captured
        let where_str = split.where_clause.to_string();
        assert!(where_str.contains("for"));
        assert!(where_str.contains("IntoIterator"));
    }

    #[test]
    fn test_add_bound() {
        let generics: Generics = parse_quote!(<T, U>);
        let bound: syn::TypeParamBound = parse_quote!(Clone);
        let result = add_bound_to_all_type_params(&generics, bound);

        // Verify bounds were added
        for param in &result.params {
            if let syn::GenericParam::Type(tp) = param {
                assert!(!tp.bounds.is_empty());
            }
        }
    }
}
