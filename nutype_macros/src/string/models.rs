use kinded::Kinded;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::common::models::{Guard, RawGuard, SpannedItem, TypeTrait, TypedCustomFunction};

// Sanitizer

pub type SpannedStringSanitizer = SpannedItem<StringSanitizer>;

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum StringSanitizer {
    Trim,
    Lowercase,
    Uppercase,
    With(TypedCustomFunction),
}

// Validator
//

pub type SpannedStringValidator = SpannedItem<StringValidator>;

#[derive(Debug, Kinded)]
#[kinded(display = "snake_case")]
pub enum StringValidator {
    CharLenMin(usize),
    CharLenMax(usize),
    NotEmpty,
    Predicate(TypedCustomFunction),
    #[cfg_attr(not(feature = "regex"), allow(dead_code))]
    Regex(RegexDef),
}

#[cfg_attr(not(feature = "regex"), allow(dead_code))]
#[derive(Debug)]
pub enum RegexDef {
    /// The case, when regex is defined with string literal inlined, e.g.:
    ///     regex = "^[0-9]{9}$"
    StringLiteral(syn::LitStr),

    /// The case, when regex is with an ident, that refers to regex constant:
    ///     regex = SSN_REGEX
    Path(syn::Path),
}

// Traits
//
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StringDeriveTrait {
    // Standard
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    FromStr,
    AsRef,
    From,
    TryFrom,
    Into,
    Hash,
    Borrow,
    Display,
    Default,
    Deref,

    // // External crates
    //
    SerdeSerialize,
    SerdeDeserialize,
    SchemarsJsonSchema,
    // Arbitrary,
}

impl TypeTrait for StringDeriveTrait {
    fn is_from_str(&self) -> bool {
        self == &Self::FromStr
    }
}

pub type StringRawGuard = RawGuard<SpannedStringSanitizer, SpannedStringValidator>;
pub type StringGuard = Guard<StringSanitizer, StringValidator>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringInnerType;

impl ToTokens for StringInnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        quote!(String).to_tokens(token_stream);
    }
}
