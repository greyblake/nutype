use crate::expand_nutype;
use proc_macro2::TokenStream;
use quote::quote;

/// Expand the same input repeatedly and assert every expansion is byte-identical.
///
/// Each collection built during expansion gets its own hasher state, so a `HashSet`
/// regression shows up as a difference between repeated expansions.
fn assert_reproducible(case: &str, attrs: TokenStream, type_definition: TokenStream) {
    let expected = expand_nutype(attrs.clone(), type_definition.clone())
        .unwrap_or_else(|err| panic!("{case}: expansion should succeed: {err}"))
        .to_string();

    for _ in 0..100 {
        let actual = expand_nutype(attrs.clone(), type_definition.clone())
            .unwrap_or_else(|err| panic!("{case}: expansion should succeed: {err}"))
            .to_string();
        assert_eq!(actual, expected, "{case}: expansion is not reproducible");
    }
}

/// `Serialize`/`Deserialize` are rejected unless the `serde` feature is on, so they are
/// only included in the derive list when the tests are run with that feature.
fn serde_derives() -> TokenStream {
    #[cfg(feature = "serde")]
    return quote!(Serialize, Deserialize,);
    #[cfg(not(feature = "serde"))]
    return TokenStream::new();
}

/// Same as [`serde_derives`], for the `arbitrary` feature.
fn arbitrary_derive() -> TokenStream {
    #[cfg(feature = "arbitrary")]
    return quote!(Arbitrary,);
    #[cfg(not(feature = "arbitrary"))]
    return TokenStream::new();
}

#[test]
fn string_expansion_is_reproducible() {
    let serde = serde_derives();
    let arbitrary = arbitrary_derive();
    assert_reproducible(
        "string",
        quote!(
            derive(
                Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Default, AsRef,
                Deref, Borrow, TryFrom, Into, FromStr, #serde #arbitrary
            ),
            default = "abc",
            sanitize(trim, lowercase),
            validate(len_char_min = 3, len_char_max = 5)
        ),
        quote!(
            struct Name(String);
        ),
    );
}

#[test]
fn integer_expansion_is_reproducible() {
    let serde = serde_derives();
    let arbitrary = arbitrary_derive();
    assert_reproducible(
        "integer",
        quote!(
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Default,
                AsRef, Deref, Borrow, TryFrom, Into, FromStr, #serde #arbitrary
            ),
            default = 5,
            validate(greater_or_equal = 1, less_or_equal = 100)
        ),
        quote!(
            struct Age(u8);
        ),
    );
}

#[test]
fn float_expansion_is_reproducible() {
    let serde = serde_derives();
    let arbitrary = arbitrary_derive();
    // No `Hash`: it cannot be derived for float-based types.
    assert_reproducible(
        "float",
        quote!(
            derive(
                Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, AsRef, Deref,
                Borrow, TryFrom, Into, FromStr, #serde #arbitrary
            ),
            validate(finite)
        ),
        quote!(
            struct Distance(f64);
        ),
    );
}

#[test]
fn any_expansion_is_reproducible() {
    assert_reproducible(
        "any",
        quote!(derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, AsRef, Deref, Borrow
        )),
        quote!(
            struct Payload(Vec<u8>);
        ),
    );
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_expansion_is_reproducible() {
    let serde = serde_derives();
    // No `Arbitrary`: it additionally requires `rust_decimal/rust-fuzz`.
    assert_reproducible(
        "decimal",
        quote!(
            derive(
                Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Default, AsRef,
                Deref, Borrow, TryFrom, Into, FromStr, #serde
            ),
            default = 1,
            validate(greater_or_equal = 0, less_or_equal = 1000)
        ),
        quote!(
            struct Price(Decimal);
        ),
    );
}

/// Conditional derives take a separate code path (`process_conditional_derives`), so they
/// need their own guard against a `HashSet` sneaking back in.
#[test]
fn conditional_derives_expansion_is_reproducible() {
    assert_reproducible(
        "cfg_attr",
        quote!(
            derive(
                Debug, Clone, PartialEq, Eq, Hash, Display, AsRef, Deref, TryFrom, Into
            ),
            cfg_attr(unix, derive(FromStr, Borrow)),
            cfg_attr(test, derive(PartialOrd, Ord, Default)),
            default = "abc",
            validate(len_char_min = 3)
        ),
        quote!(
            struct Name(String);
        ),
    );
}
