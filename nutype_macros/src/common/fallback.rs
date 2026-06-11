//! Best-effort recovery used when `#[nutype(...)]` fails to expand.
//!
//! `nutype` consumes the annotated struct and re-emits a brand new
//! `struct Name(Inner);`. When expansion fails (most commonly because the user
//! is still *typing* the attribute arguments and they don't parse yet), the
//! macro would otherwise emit only `compile_error!(...)`, which makes the type
//! itself vanish from the compiler's and rust-analyzer's view. Every downstream
//! `Name::...`, `let x: Name`, and field access then turns red, even though the
//! only thing wrong is an unfinished attribute.
//!
//! To keep rust-analyzer resilient, the error path emits the skeleton produced
//! here *alongside* the real `compile_error!`. The skeleton only needs to
//! type-check; it does not enforce any invariants.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Best-effort, type-checkable skeleton of the newtype.
///
/// Returns `None` when we cannot even recover a single-field tuple struct from
/// the input (in that case the caller emits only the compile error).
pub fn fallback_skeleton(type_definition: &TokenStream) -> Option<TokenStream> {
    let input: DeriveInput = syn::parse2(type_definition.clone()).ok()?;

    let DeriveInput {
        vis,
        ident,
        generics,
        data,
        ..
    } = input;

    let data_struct = match data {
        Data::Struct(s) => s,
        _ => return None,
    };

    let inner_ty = match data_struct.fields {
        Fields::Unnamed(fields) => fields.unnamed.into_iter().next()?.ty,
        _ => return None,
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Some(quote! {
        // NB: the inner field visibility is intentionally dropped. The real
        // expansion forbids a visible inner field; the skeleton only needs to
        // type-check, not to enforce that invariant.
        #vis struct #ident #generics (#inner_ty) #where_clause;

        // Stub the inherent methods nutype always generates (the constructors and
        // `into_inner`) so that completion and resolution on `Name::...` keep
        // working while the attribute is broken. We can't know whether the real
        // type will end up validated (`try_new`) or not (`new`), so we provide
        // both. `unimplemented!()` diverges and coerces to any return type, the
        // argument is taken via `impl Into<Inner>` (every type satisfies the
        // reflexive `Inner: Into<Inner>`), and no extra type is introduced, so
        // this type-checks for every inner type without polluting the namespace.
        #[doc(hidden)]
        #[allow(dead_code, unused_variables, clippy::all)]
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn new(raw_value: impl ::core::convert::Into<#inner_ty>) -> Self {
                ::core::unimplemented!()
            }

            pub fn try_new(
                raw_value: impl ::core::convert::Into<#inner_ty>,
            ) -> ::core::result::Result<Self, ::core::convert::Infallible> {
                ::core::unimplemented!()
            }

            pub fn into_inner(self) -> #inner_ty {
                ::core::unimplemented!()
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renders(ts: &TokenStream) -> String {
        ts.to_string()
    }

    #[test]
    fn skeleton_for_simple_string_newtype() {
        let input = quote! { pub struct Foo(String); };
        let out = fallback_skeleton(&input).expect("should produce a skeleton");

        // The output must be valid Rust.
        syn::parse2::<syn::File>(out.clone()).expect("skeleton must parse as a file");

        let rendered = renders(&out);
        assert!(rendered.contains("struct Foo"));
        assert!(rendered.contains("String"));
        // All the inherent methods nutype always generates are stubbed so that
        // `Foo::new(..)`, `Foo::try_new(..)` and `foo.into_inner()` keep
        // resolving while the attribute is broken.
        assert!(rendered.contains("fn new"));
        assert!(rendered.contains("fn try_new"));
        assert!(rendered.contains("fn into_inner"));
    }

    #[test]
    fn skeleton_preserves_generics_and_where_clause() {
        let input = quote! { pub struct Wrapper<T: Clone>(T) where T: Default; };
        let out = fallback_skeleton(&input).expect("should produce a skeleton");

        syn::parse2::<syn::File>(out.clone()).expect("skeleton must parse as a file");

        let rendered = renders(&out);
        assert!(rendered.contains("struct Wrapper"));
        assert!(rendered.contains("Clone"));
        assert!(rendered.contains("Default"));
    }

    #[test]
    fn no_skeleton_for_named_struct() {
        let input = quote! { struct S { x: u8 } };
        assert!(fallback_skeleton(&input).is_none());
    }

    #[test]
    fn no_skeleton_for_enum() {
        let input = quote! { enum E { A, B } };
        assert!(fallback_skeleton(&input).is_none());
    }

    #[test]
    fn no_skeleton_for_empty_tuple_struct() {
        let input = quote! { struct S(); };
        assert!(fallback_skeleton(&input).is_none());
    }

    #[test]
    fn no_skeleton_for_unparsable_input() {
        let input = quote! { this is not a struct @ # !; };
        assert!(fallback_skeleton(&input).is_none());
    }
}
