pub mod error;
pub mod new_unchecked;
pub mod parse_error;
pub mod tests;
pub mod traits;

use core::hash::Hash;
use std::collections::HashSet;

use self::traits::GeneratedTraits;

use super::models::{
    ConstFn, CustomFunction, ErrorTypePath, GenerateParams, Guard, NewUnchecked,
    ParseErrorTypeName, SpannedDeriveUnsafeTrait, TypeName, TypeTrait,
};
use crate::common::{
    generate::{new_unchecked::gen_new_unchecked, parse_error::gen_parse_error_name},
    models::{ModuleName, Validation},
};
use proc_macro2::{Punct, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{Generics, Visibility};

/// Inject an inner type into a closure, so compiler does not complain if the token stream matchers
/// the expected closure pattern.
///
/// Input:
///   |s| s.trim().to_lowercase()
/// Output:
///   |s: String| s.trim().to_lowercase()
///
/// or
///
/// Input:
///   |mut s| s.trim().to_lowercase()
/// Output:
///   |mut s: String| s.trim().to_lowercase()
// TODO: consider using syn instead messing with TokenStream directly
pub fn type_custom_closure(
    closure_or_func_path: &TokenStream,
    inner_type: impl ToTokens,
) -> TokenStream {
    let inner_type_tokens = quote!(#inner_type);
    let mut ts: Vec<TokenTree> = closure_or_func_path.clone().into_iter().collect();

    if ts.len() >= 3 && is_pipe(&ts[0]) && is_ident(&ts[1]) && is_pipe(&ts[2]) {
        // If the tokens match `|s|` pattern
        // then inject the type, e.g. `|s: String|`
        insert_type_at_position(&mut ts, inner_type_tokens, 2);
    } else if ts.len() >= 4
        && is_pipe(&ts[0])
        && is_mut(&ts[1])
        && is_ident(&ts[2])
        && is_pipe(&ts[3])
    {
        // If the tokens match `|mut s|` pattern,
        // then inject the type, e.g. `|mut s: String|`
        insert_type_at_position(&mut ts, inner_type_tokens, 3);
    }

    ts.into_iter().collect()
}

fn insert_type_at_position(ts: &mut Vec<TokenTree>, inner_type: TokenStream, pos: usize) {
    // Insert `:`
    let colon = TokenTree::Punct(Punct::new(':', Spacing::Alone));
    ts.insert(pos, colon);

    // Insert tokens of the type at position `pos +1` (basically after `:`)
    for (index, tok) in inner_type.into_iter().enumerate() {
        let pos = pos + 1 + index;
        ts.insert(pos, tok);
    }
}

fn is_pipe(token: &TokenTree) -> bool {
    match token {
        TokenTree::Punct(punct) => punct.as_char() == '|',
        _ => false,
    }
}

fn is_ident(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Ident(_))
}

fn is_mut(token: &TokenTree) -> bool {
    match token {
        TokenTree::Ident(ident) => ident == "mut",
        _ => false,
    }
}

pub fn gen_module_name_for_type(type_name: &TypeName) -> ModuleName {
    let ident = format_ident!("__nutype_{type_name}__");
    ModuleName::new(ident)
}

pub fn gen_reimports(
    vis: Visibility,
    type_name: &TypeName,
    module_name: &ModuleName,
    maybe_error_type_path: Option<&ErrorTypePath>,
    maybe_parse_error_type_name: Option<&ParseErrorTypeName>,
) -> TokenStream {
    let reimport_main_type = quote! {
        #vis use #module_name::#type_name;
    };

    let reimport_error_type_if_needed = match maybe_error_type_path {
        None => quote!(),
        Some(ref error_type_path) => {
            quote! (
                #vis use #module_name::#error_type_path;
            )
        }
    };

    let reimport_parse_error_type_if_needed = match maybe_parse_error_type_name {
        None => quote!(),
        Some(ref parse_error_type_path) => {
            quote! (
                #vis use #module_name::#parse_error_type_path;
            )
        }
    };

    quote! {
        #reimport_main_type
        #reimport_error_type_if_needed
        #reimport_parse_error_type_if_needed
    }
}

pub fn gen_impl_into_inner(
    type_name: &TypeName,
    generics: &Generics,
    inner_type: impl ToTokens,
    const_fn: ConstFn,
) -> TokenStream {
    let generics_without_bounds = strip_trait_bounds_on_generics(generics);
    quote! {
        impl #generics #type_name #generics_without_bounds {
            #[inline]
            pub #const_fn fn into_inner(self) -> #inner_type {
                self.0
            }
        }
    }
}

/// Remove trait bounds from generics.
///
/// Input:
///    <T: Display + Debug, U: Clone>
///
/// Output:
///   <T, U>
pub fn strip_trait_bounds_on_generics(original: &Generics) -> Generics {
    let mut generics = original.clone();
    for param in &mut generics.params {
        if let syn::GenericParam::Type(syn::TypeParam { bounds, .. }) = param {
            *bounds = syn::punctuated::Punctuated::new();
        }
    }
    generics
}

/// Add a bound to all generics types.
///
/// Input:
///     <T, U>
///     Serialize
///
/// Output:
///    <T: Serialize, U: Serialize>
pub fn add_bound_to_all_type_params(generics: &Generics, bound: TokenStream) -> Generics {
    let mut generics = generics.clone();
    let parsed_bound: syn::TypeParamBound =
        syn::parse2(bound).expect("Failed to parse TypeParamBound");
    for param in &mut generics.params {
        if let syn::GenericParam::Type(syn::TypeParam { bounds, .. }) = param {
            bounds.push(parsed_bound.clone());
        }
    }
    generics
}

/// Add a parameter to generics.
///
/// Input:
///     <T, U>
///     'a
///
/// Output:
///     <'a, T, U>
///
pub fn add_param(generics: &Generics, param: TokenStream) -> Generics {
    let mut generics = generics.clone();
    let parsed_param: syn::GenericParam = syn::parse2(param).expect("Failed to parse GenericParam");
    generics.params.push(parsed_param);
    generics
}

pub trait GenerateNewtype {
    type Sanitizer;
    type Validator;
    type InnerType: ToTokens;
    type TypedTrait: Hash + TypeTrait;

    /// If the type has dedicated parse error. This error is used within `FromStr` trait.
    /// For most of the types it's different from the regular validation error, because parsing
    /// happens in 2 stages:
    /// * &str -> inner type (parsing)
    /// * inner type -> nutype (validation)
    ///
    /// But for the String based types there is no first stage, so the parse error is the same as
    /// validation error.
    const HAS_DEDICATED_PARSE_ERROR: bool = true;

    /// If it's true, then `::new()` function receives `impl Into<T>` instead of `T`.
    const NEW_CONVERT_INTO_INNER_TYPE: bool = false;

    fn gen_fn_sanitize(
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
        const_fn: ConstFn,
    ) -> TokenStream;

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
        const_fn: ConstFn,
    ) -> TokenStream;

    fn gen_validation_error_type(
        type_name: &TypeName,
        error_type_path: &ErrorTypePath,
        validators: &[Self::Validator],
    ) -> TokenStream;

    fn gen_traits(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        traits: HashSet<Self::TypedTrait>,
        unsafe_traits: &[SpannedDeriveUnsafeTrait],
        maybe_default_value: Option<syn::Expr>,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
    ) -> Result<GeneratedTraits, syn::Error>;

    fn gen_try_new(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
        validation: &Validation<Self::Validator>,
        const_fn: ConstFn,
    ) -> TokenStream {
        let generics_without_bounds = strip_trait_bounds_on_generics(generics);
        let fn_sanitize = Self::gen_fn_sanitize(inner_type, sanitizers, const_fn);

        let maybe_generated_validation_error = match validation {
            Validation::Standard {
                validators,
                error_type_path,
            } => {
                let validation_error =
                    Self::gen_validation_error_type(type_name, error_type_path, validators);
                Some(validation_error)
            }
            Validation::Custom { .. } => None,
        };

        let fn_validate = match validation {
            Validation::Standard {
                validators,
                error_type_path,
            } => Self::gen_fn_validate(inner_type, error_type_path, validators, const_fn),
            Validation::Custom {
                with,
                error_type_path,
            } => gen_fn_validate_custom(inner_type, with, error_type_path, const_fn),
        };

        let (input_type, convert_raw_value_if_necessary) = if Self::NEW_CONVERT_INTO_INNER_TYPE {
            (
                quote!(impl Into<#inner_type>),
                quote!(let raw_value = raw_value.into();),
            )
        } else {
            (quote!(#inner_type), quote!())
        };

        let error_type_path = validation.error_type_path();

        quote!(
            #maybe_generated_validation_error

            impl #generics #type_name #generics_without_bounds {
                pub #const_fn fn try_new(raw_value: #input_type) -> ::core::result::Result<Self, #error_type_path> {
                    #convert_raw_value_if_necessary

                    let sanitized_value: #inner_type = Self::__sanitize__(raw_value);
                    // NOTE:  `?` operator cannot be used in const functions: https://github.com/rust-lang/rust/issues/74935
                    // So we cannot write
                    //     Self::__validate__(&sanitized_value)?;
                    #[allow(clippy::question_mark)]
                    if let Err(e) = Self::__validate__(&sanitized_value) {
                        return Err(e);
                    }
                    Ok(#type_name(sanitized_value))
                }

                // Definite associated private functions __sanitize__() and __validate__() with underscores so they do not overlap with outer
                // scope imported with `use super::*`.
                #fn_sanitize
                #fn_validate
            }
        )
    }

    fn gen_new(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
        const_fn: ConstFn,
    ) -> TokenStream {
        let generics_without_bounds = strip_trait_bounds_on_generics(generics);
        let fn_sanitize = Self::gen_fn_sanitize(inner_type, sanitizers, const_fn);

        let (input_type, convert_raw_value_if_necessary) = if Self::NEW_CONVERT_INTO_INNER_TYPE {
            (
                quote!(impl Into<#inner_type>),
                quote!(let raw_value = raw_value.into();),
            )
        } else {
            (quote!(#inner_type), quote!())
        };

        quote!(
            impl #generics #type_name #generics_without_bounds {
                pub #const_fn fn new(raw_value: #input_type) -> Self {
                    #convert_raw_value_if_necessary
                    Self(Self::__sanitize__(raw_value))
                }
                // Definite associated private function __sanitize__() with underscores so they do not overlap with outer
                // scope imported with `use super::*`.
                #fn_sanitize
            }
        )
    }

    fn gen_implementation(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        new_unchecked: NewUnchecked,
        const_fn: ConstFn,
    ) -> TokenStream {
        let impl_new = match guard {
            Guard::WithoutValidation { sanitizers } => {
                Self::gen_new(type_name, generics, inner_type, sanitizers, const_fn)
            }
            Guard::WithValidation {
                sanitizers,
                validation,
            } => Self::gen_try_new(
                type_name, generics, inner_type, sanitizers, validation, const_fn,
            ),
        };
        let impl_into_inner = gen_impl_into_inner(type_name, generics, inner_type, const_fn);
        let impl_new_unchecked = gen_new_unchecked(type_name, inner_type, new_unchecked, const_fn);

        quote! {
            #impl_new
            #impl_into_inner
            #impl_new_unchecked
        }
    }

    #[allow(clippy::type_complexity)]
    fn gen_nutype(
        params: GenerateParams<
            Self::InnerType,
            Self::TypedTrait,
            Guard<Self::Sanitizer, Self::Validator>,
        >,
    ) -> Result<TokenStream, syn::Error> {
        let GenerateParams {
            doc_attrs,
            traits,
            unsafe_traits,
            vis,
            type_name,
            guard,
            new_unchecked,
            const_fn,
            maybe_default_value,
            inner_type,
            generics,
        } = params;

        let module_name = gen_module_name_for_type(&type_name);
        let implementation = Self::gen_implementation(
            &type_name,
            &generics,
            &inner_type,
            &guard,
            new_unchecked,
            const_fn,
        );

        let has_from_str_trait = traits.iter().any(|t| t.is_from_str());
        let maybe_parse_error_type_path = if has_from_str_trait && Self::HAS_DEDICATED_PARSE_ERROR {
            Some(gen_parse_error_name(&type_name))
        } else {
            None
        };

        let tests = Self::gen_tests(
            &type_name,
            &generics,
            &inner_type,
            &maybe_default_value,
            &guard,
            &traits,
        );

        let maybe_reimported_error_type_path = match &guard {
            Guard::WithoutValidation { .. } => None,
            Guard::WithValidation { validation, .. } => match validation {
                // We won't need to reimport error if it's a custom error provided by the user.
                Validation::Custom { .. } => None,
                Validation::Standard {
                    error_type_path, ..
                } => Some(error_type_path),
            },
        };

        let reimports = gen_reimports(
            vis,
            &type_name,
            &module_name,
            maybe_reimported_error_type_path,
            maybe_parse_error_type_path.as_ref(),
        );

        let GeneratedTraits {
            derive_transparent_traits,
            implement_traits,
        } = Self::gen_traits(
            &type_name,
            &generics,
            &inner_type,
            traits,
            &unsafe_traits,
            maybe_default_value,
            &guard,
        )?;

        Ok(quote!(
            #[doc(hidden)]
            #[allow(non_snake_case, reason = "we keep original structure name which is probably CamelCase")]
            mod #module_name {
                use super::*;

                #(#doc_attrs)*
                #derive_transparent_traits
                pub struct #type_name #generics(#inner_type);

                #implementation
                #implement_traits

                #[cfg(test)]
                mod tests {
                    use super::*;
                    #tests
                }
            }
            #reimports
        ))
    }

    fn gen_tests(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        maybe_default_value: &Option<syn::Expr>,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        traits: &HashSet<Self::TypedTrait>,
    ) -> TokenStream;
}

fn gen_fn_validate_custom<InnerType: ToTokens>(
    inner_type: &InnerType,
    with: &CustomFunction,
    error_type_path: &ErrorTypePath,
    const_fn: ConstFn,
) -> TokenStream {
    quote! {
        // For some types like `String` clippy suggests using `&str` instead of `&String` here,
        // but it does not really matter in this context.
        #[allow(clippy::ptr_arg)]
        #const_fn fn __validate__(value: &#inner_type) -> ::core::result::Result<(), #error_type_path> {
            #with(value)
        }
    }
}
