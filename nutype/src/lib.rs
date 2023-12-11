//! <p align="center"><img width="300" src="https://raw.githubusercontent.com/greyblake/nutype/master/art/rust_nutype.png" alt="Rust Nutype Logo"></p>
//! <h2 align="center">The newtype with guarantees.</h2>
//!
//! Nutype is a proc macro that allows adding extra constraints like _sanitization_ and _validation_ to the regular [newtype pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html). The generated code makes it impossible to instantiate a value without passing the checks. It works this way even with `serde` deserialization.
//!
//!
//! ## Quick start
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(
//!     sanitize(trim, lowercase),
//!     validate(not_empty, len_char_max = 20),
//!     derive(Debug, PartialEq),
//! )]
//! pub struct Username(String);
//!
//! // Now we can create usernames:
//! assert_eq!(
//!     Username::new("   FooBar  ").unwrap().into_inner(),
//!     "foobar"
//! );
//!
//! // But we cannot create invalid ones:
//! assert_eq!(
//!     Username::new("   "),
//!     Err(UsernameError::NotEmptyViolated),
//! );
//!
//! assert_eq!(
//!     Username::new("TheUserNameIsVeryVeryLong"),
//!     Err(UsernameError::LenCharMaxViolated),
//! );
//! ```
//!
//! Note, that we also got `UsernameError` enum generated implicitly.
//!
//! Ok, but let's try to obtain an instance of `Username` that violates the validation rules:
//!
//! ```ignore
//! let username = Username("".to_string())
//!
//! // error[E0423]: cannot initialize a tuple struct which contains private fields
//! ```
//!
//! ```ignore
//! let mut username = Username::new("foo").unwrap();
//! username.0 = "".to_string();
//!
//! // error[E0616]: field `0` of struct `Username` is private
//! ```
//!
//! Haha. It's does not seem to be easy!
//!
//!
//! ## A few more examples
//!
//! Here are some other examples of what you can do with `nutype`.
//!
//! You can skip `sanitize` and use a custom validator `with`:
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(validate(predicate = |n| n % 2 == 1))]
//! struct OddNumber(i64);
//! ```
//!
//! You can skip validation, if you need sanitization only:
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(sanitize(trim, lowercase))]
//! struct Username(String);
//! ```
//!
//! In that case, `Username::new(String)` simply returns `Username`, not `Result`.
//!
//!
//! ## Inner types
//!
//! Available sanitizers, validators, and derivable traits are determined by the inner type, which falls into the following categories:
//! * String
//! * Integer (`u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`)
//! * Float (`f32`, `f64`)
//! * Any other arbitrary type
//!
//! ## String
//!
//! At the moment the string inner type supports only `String` (owned) type.
//!
//! ### String sanitizers
//!
//! | Sanitizer   | Description                                                                         | Example                                         |
//! |-------------|-------------------------------------------------------------------------------------|-------------------------------------------------|
//! | `trim`      | Removes leading and trailing whitespaces                                            | `trim`                                          |
//! | `lowercase` | Converts the string to lowercase                                                    | `lowercase`                                     |
//! | `uppercase` | Converts the string to uppercase                                                    | `uppercase`                                     |
//! | `with`      | Custom sanitizer. A function or closure that receives `String` and returns `String` | `with = \|mut s: String\| { s.truncate(5); s }` |
//!
//! ### String validators
//!
//! | Validator      | Description                                                                     | Error variant        | Example                                      |
//! |----------------|---------------------------------------------------------------------------------|----------------------|----------------------------------------------|
//! | `len_char_min` | Min length of the string (in chars, not bytes)                                  | `LenCharMinViolated` | `len_char_min = 5`                           |
//! | `len_char_max` | Max length of the string (in chars, not bytes)                                  | `LenCharMaxViolated` | `len_char_max = 255`                         |
//! | `not_empty`    | Rejects an empty string                                                         | `NotEmptyViolated`   | `not_empty`                                  |
//! | `regex`        | Validates format with a regex. Requires `regex` feature.                        | `RegexViolated`      | `regex = "^[0-9]{7}$"` or `regex = ID_REGEX` |
//! | `predicate`    | Custom validator. A function or closure that receives `&str` and returns `bool` | `PredicateViolated`  | `predicate = \|s: &str\| s.contains('@')`    |
//!
//! #### Regex validation
//!
//! Requirements:
//! * `regex` feature of `nutype` is enabled.
//! * You crate have to explicitly include `regex` and `lazy_static` dependencies.
//!
//! There are a number of ways you can use regex.
//!
//! A regular expression can be defined right in place:
//!
//! ```ignore
//! use nutype::nutype;
//!
//! #[nutype(validate(regex = "^[0-9]{3}-[0-9]{3}$"))]
//! pub struct PhoneNumber(String);
//! ```
//!
//! or it can be defined with `lazy_static`:
//!
//! ```ignore
//! use nutype::nutype;
//! use lazy_static::lazy_static;
//! use regex::Regex;
//!
//! lazy_static! {
//!     static ref PHONE_NUMBER_REGEX: Regex = Regex::new("^[0-9]{3}-[0-9]{3}$").unwrap();
//! }
//!
//! #[nutype(validate(regex = PHONE_NUMBER_REGEX))]
//! pub struct PhoneNumber(String);
//! ```
//!
//! or `once_cell`:
//!
//! ```ignore
//! use nutype::nutype;
//! use once_cell::sync::Lazy;
//! use regex::Regex;
//!
//! static PHONE_NUMBER_REGEX: Lazy<Regex> =
//!     Lazy::new(|| Regex::new("[0-9]{3}-[0-9]{3}$").unwrap());
//!
//! #[nutype(validate(regex = PHONE_NUMBER_REGEX))]
//! pub struct PhoneNumber(String);
//! ```
//!
//!
//! ### String derivable traits
//!
//! The following traits can be derived for a string-based type:
//! `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
//! `From`, `TryFrom`, `Into`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.
//!
//!
//! ## Integer
//!
//! The integer inner types are: `u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`.
//!
//! ### Integer sanitizers
//!
//! | Sanitizer | Description       | Example                            |
//! |-----------|-------------------|------------------------------------|
//! | `with`    | Custom sanitizer. | `with = \|raw\| raw.clamp(0, 100)` |
//!
//! ### Integer validators
//!
//! | Validator           | Description           | Error variant             | Example                              |
//! | ------------------- | --------------------- | ------------------------- | ------------------------------------ |
//! | `less`              | Exclusive upper bound | `LessViolated`            | `less = 100`                         |
//! | `less_or_equal`     | Inclusive upper bound | `LessOrEqualViolated`     | `less_or_equal = 99`                 |
//! | `greater`           | Exclusive lower bound | `GreaterViolated`         | `greater = 17`                       |
//! | `greater_or_equal`  | Inclusive lower bound | `GreaterOrEqualViolated`  | `greater_or_equal = 18`              |
//! | `predicate`         | Custom predicate      | `PredicateViolated`       | `predicate = \|num\| num % 2 == 0`   |
//!
//! ### Integer derivable traits
//!
//! The following traits can be derived for an integer-based type:
//! `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
//! `Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.
//!
//!
//! ## Float
//!
//! The float inner types are: `f32`, `f64`.
//!
//! ### Float sanitizers
//!
//! | Sanitizer | Description       | Example                                |
//! |-----------|-------------------|----------------------------------------|
//! | `with`    | Custom sanitizer. | `with = \|val\| val.clamp(0.0, 100.0)` |
//!
//! ### Float validators
//!
//! | Validator          | Description                      | Error variant            | Example                             |
//! | ------------------ | -------------------------------- | ---------------------    | ----------------------------------- |
//! | `less`             | Exclusive upper bound            | `LessViolated`           | `less = 100.0`                      |
//! | `less_or_equal`    | Inclusive upper bound            | `LessOrEqualViolated`    | `less_or_equal = 100.0`             |
//! | `greater`          | Exclusive lower bound            | `GreaterViolated`        | `greater = 0.0`                     |
//! | `greater_or_equal` | Inclusive lower bound            | `GreaterOrEqualViolated` | `greater_or_equal = 0.0`            |
//! | `finite`           | Check against NaN and infinity   | `FiniteViolated`         | `finite`                            |
//! | `predicate`        | Custom predicate                 | `PredicateViolated`      | `predicate = \|val\| val != 50.0`   |
//!
//! ### Float derivable traits
//!
//! The following traits can be derived for a float-based type:
//! `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
//! `Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.
//!
//! It's also possible to derive `Eq` and `Ord` if the validation rules guarantee that `NaN` is excluded.
//! This can be done by applying `finite` validation. For example:
//!
//! ```rust
//! use nutype::nutype;
//!
//! #[nutype(
//!     validate(finite),
//!     derive(PartialEq, Eq, PartialOrd, Ord),
//! )]
//! struct Size(f64);
//! ```
//!
//! ## Other inner types
//!
//! For any other type it is possible to define custom sanitizers with `with` and custom
//! validations with `predicate`:
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(
//!     derive(Debug, PartialEq, Deref, AsRef),
//!     sanitize(with = |mut guests| { guests.sort(); guests }),
//!     validate(predicate = |guests| !guests.is_empty() ),
//! )]
//! pub struct GuestList(Vec<String>);
//!
//! // Empty list is not allowed
//! assert_eq!(
//!     GuestList::new(vec![]),
//!     Err(GuestListError::PredicateViolated)
//! );
//!
//! // Create the list of our guests
//! let guest_list = GuestList::new(vec![
//!     "Seneca".to_string(),
//!     "Marcus Aurelius".to_string(),
//!     "Socrates".to_string(),
//!     "Epictetus".to_string(),
//! ]).unwrap();
//!
//! // The list is sorted (thanks to sanitize)
//! assert_eq!(
//!     guest_list.as_ref(),
//!     &[
//!         "Epictetus".to_string(),
//!         "Marcus Aurelius".to_string(),
//!         "Seneca".to_string(),
//!         "Socrates".to_string(),
//!     ]
//! );
//!
//! // Since GuestList derives Deref, we can use methods from `Vec<T>`
//! // due to deref coercion (if it's a good idea or not, it's left up to you to decide!).
//! assert_eq!(guest_list.len(), 4);
//!
//! for guest in guest_list.iter() {
//!     println!("{guest}");
//! }
//!
//! ```
//!
//! ## Custom sanitizers
//!
//! You can set custom sanitizers using the `with` option.
//! A custom sanitizer is a function or closure that receives a value of an inner type with ownership and returns a sanitized value back.
//!
//! For example, this one
//!
//! ```ignore
//! use nutype::nutype;
//!
//! fn new_to_old(s: String) -> String {
//!     s.replace("New", "Old")
//! }
//!
//! #[nutype(sanitize(with = new_to_old))]
//! struct CityName(String);
//! ```
//!
//! is equal to the following one:
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(sanitize(with = |s| s.replace("New", "Old") ))]
//! struct CityName(String);
//!
//! // And works the same way:
//! let city = CityName::new("New York");
//! assert_eq!(city.into_inner(), "Old York");
//! ```
//!
//! ## Custom validators
//!
//! In similar fashion it's possible to define custom validators, but a validation function receives a reference and returns `bool`.
//! Think of it as a predicate.
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(validate(predicate = is_valid_name))]
//! pub struct Name(String);
//!
//! fn is_valid_name(name: &str) -> bool {
//!     // A fancy way to verify if the first character is uppercase
//!     name.chars().next().map(char::is_uppercase).unwrap_or(false)
//! }
//!
//! fn main() { }
//! ```
//!
//! ## Recipes
//!
//! ### Derive `Default`
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(
//!     derive(Default),
//!     default = "Anonymous",
//! )]
//! pub struct Name(String);
//! ```
//!
//! ### Derive `Eq` and `Ord` on float types
//!
//! With nutype it's possible to derive `Eq` and `Ord` if there is `finite` validation set.
//! The `finite` validation ensures that the valid value excludes `NaN`.
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype(
//!     validate(finite),
//!     derive(PartialEq, Eq, PartialOrd, Ord),
//! )]
//! pub struct Weight(f64);
//! ```
//!
//! ## How to break the constraints?
//!
//! It's discouraged, but it's possible to bypass the constraints by enabling `new_unchecked` crate feature and marking a type with `new_unchecked`:
//!
//! ```ignore
//! use nutype::nutype;
//!
//! #[nutype(
//!     new_unchecked
//!     sanitize(trim)
//!     validate(len_char_min = 8)
//! )]
//! pub struct Name(String);
//!
//! // Yes, you're forced to use `unsafe` here, so everyone will point fingers at YOU.
//! let name = unsafe { Name::new_unchecked(" boo ".to_string()) };
//!
//! // `name` violates the sanitization and validation rules!!!
//! assert_eq!(name.into_inner(), " boo ");
//! ```
//!
//! ## Feature flags
//!
//! * `serde` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.
//! * `new_unchecked` - enables generation of unsafe `::new_unchecked()` function.
//! * `schemars08` - allows to derive [`JsonSchema`](https://docs.rs/schemars/0.8.12/schemars/trait.JsonSchema.html) trait of [schemars](https://crates.io/crates/schemars) crate. Note that at the moment validation rules are not respected.
//! * `regex` - allows to use `regex = ` validation on string-based types. Note: your crate also has to explicitly have `regex` and `lazy_static` within dependencies.
//!
//! ## Support Ukrainian military forces 🇺🇦
//!
//! Today I live in Berlin, I have the luxury to live a physically safe life.
//! But I am Ukrainian. The first 25 years of my life I spent in [Kharkiv](https://en.wikipedia.org/wiki/Kharkiv),
//! the second-largest city in Ukraine, 60km away from the border with russia. Today about [a third of my home city is destroyed](https://www.youtube.com/watch?v=ihoufBFSZds) by russians.
//! My parents, my relatives and my friends had to survive the artillery and air attack, living for over a month in basements.
//!
//! Some of them have managed to evacuate to EU. Some others are trying to live "normal lives" in Kharkiv, doing there daily duties.
//! And some are at the front line right now, risking their lives every second to protect the rest.
//!
//! I encourage you to donate to [Charity foundation of Serhiy Prytula](https://prytulafoundation.org/en).
//! Just pick the project you like and donate. This is one of the best-known foundations, you can watch a [little documentary](https://www.youtube.com/watch?v=VlmWqoeub1Q) about it.
//! Your contribution to the Ukrainian military force is a contribution to my calmness, so I can spend more time developing the project.
//!
//! Thank you.

pub use nutype_macros::nutype;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_example() {
        #[nutype(
            sanitize(trim, lowercase),
            validate(not_empty),
            derive(
                TryFrom, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
            )
        )]
        pub struct Email(String);

        let email = Email::new("  OH@my.example\n\n").unwrap();
        assert_eq!(email.into_inner(), "oh@my.example");

        assert_eq!(Email::new("  \n"), Err(EmailError::NotEmptyViolated));
    }

    #[test]
    fn test_amount_example() {
        #[nutype(
            validate(greater_or_equal = 100, less_or_equal = 1_000),
            derive(Debug, PartialEq, TryFrom)
        )]
        pub struct Amount(u32);

        assert_eq!(
            Amount::try_from(99),
            Err(AmountError::GreaterOrEqualViolated)
        );
        assert_eq!(
            Amount::try_from(1_001),
            Err(AmountError::LessOrEqualViolated)
        );

        assert_eq!(Amount::try_from(100).unwrap().into_inner(), 100);
    }
}
