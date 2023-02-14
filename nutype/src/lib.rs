//! <p align="center"><img width="300" src="https://raw.githubusercontent.com/greyblake/nutype/master/art/rust_nutype.png" alt="Rust Nutype Logo"></p>
//! <h2 align="center">The newtype with guarantees.</h2>
//!
//! ## Philosophy
//!
//! Nutype embraces the simple idea: **the type system can be leveraged to track the fact that something was done, so there is no need to do it again**.
//!
//! If a piece of data was once sanitized and validated we can rely on the types instead of sanitizing and validating again and again.
//!
//! ## Quick start
//!
//! ```rust
//! use nutype::nutype;
//!
//! #[nutype(
//!     sanitize(trim, lowercase)
//!     validate(not_empty, max_len = 20)
//! )]
//! pub struct Username(String);
//! ```
//!
//! Now we can create usernames:
//!
//! ```ignore
//! assert_eq!(
//!     Username::new("   FooBar  ").unwrap().into_inner(),
//!     "foobar"
//! );
//! ```
//!
//! But we cannot create invalid ones:
//!
//! ```ignore
//! assert_eq!(
//!     Username::new("   "),
//!     Err(UsernameError::Missing),
//! );
//!
//! assert_eq!(
//!     Username::new("TheUserNameIsVeryVeryLong"),
//!     Err(UsernameError::TooLong),
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
//! #[nutype(validate(with = |n| n % 2 == 1))]
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
//! You can derive traits. A lot of traits! For example:
//!
//! ```
//! use nutype::nutype;
//!
//! #[nutype]
//! #[derive(*)]
//! struct Username(String);
//! ```
//!
//! The code above derives the following traits for `Username`: `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Hash`.
//! `*` is just a syntax sugar for "derive whatever makes sense to derive by default", which is very subjective and opinionated. It's rather an experimental feature that was born
//! from the fact that `#[nutype]` has to mess with `#[derive]` anyway because users are not supposed to be able to derive traits like `DerefMut` or `BorrowMut`.
//! That would allow mutating the inner (protected) value which undermines the entire idea of nutype.
//!
//!
//! ## Inner types
//!
//! Available sanitizers, validators, and derivable traits are determined by the inner type, which falls into the following categories:
//! * String
//! * Integer (`u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`)
//! * Float (`f32`, `f64`)
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
//! | Validator   | Description                                                                     | Error variant | Example                              |
//! |-------------|---------------------------------------------------------------------------------|---------------|--------------------------------------|
//! | `max_len`   | Max length of the string                                                        | `TooLong`     | `max_len = 255`                      |
//! | `min_len`   | Min length of the string                                                        | `TooShort`    | `min_len = 5`                        |
//! | `not_empty` | Rejects an empty string                                                         | `Missing`     | `not_empty`                          |
//! | `with`      | Custom validator. A function or closure that receives `&str` and returns `bool` | `Invalid`     | `with = \|s: &str\| s.contains('@')` |
//!
//! ### String derivable traits
//!
//! The following traits can be derived for a string-based type:
//! `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `From`, `TryFrom`, `Into`, `Hash`, `Borrow`, `Display`, `Serialize`, `Deserialize`.
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
//! | Validator | Description         | Error variant | Example                       |
//! |-----------|---------------------|---------------|-------------------------------|
//! | `max`     | Maximum valid value | `TooBig`      | `max = 99`                    |
//! | `min`     | Minimum valid value | `TooSmall`    | `min = 18`                    |
//! | `with`    | Custom validator    | `Invalid`     | `with = \|num\| num % 2 == 0` |
//!
//! ### Integer derivable traits
//!
//! The following traits can be derived for an integer-based type:
//! `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Serialize`, `Deserialize`.
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
//! | Validator | Description         | Error variant | Example                       |
//! |-----------|---------------------|---------------|-------------------------------|
//! | `max`     | Maximum valid value | `TooBig`      | `max = 100.0`                 |
//! | `min`     | Minimum valid value | `TooSmall`    | `min = 0.0`                   |
//! | `with`    | Custom validator    | `Invalid`     | `with = \|val\| val != 50.0`  |
//!
//! ### Float derivable traits
//!
//! The following traits can be derived for a float-based type:
//! `Debug`, `Clone`, `Copy`, `PartialEq`, `PartialOrd`, `FromStr`, `AsRef`, `Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Serialize`, `Deserialize`.
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
//! ```
//!
//! And works the same way:
//!
//! ```ignore
//! let city = CityName::new("New York");
//! assert_eq!(city.into_inner(), "Old York");
//! ```
//!
//! ## Custom validators
//!
//! In similar fashion it's possible to define custom validators, but a validation function receives a reference and returns `bool`.
//! Think of it as a predicate.
//!
//! ```ignore
//! use nutype::nutype;
//!
//! #[nutype(validate(with = is_valid_name))]
//! pub struct Name(String);
//!
//! fn is_valid_name(name: &str) -> bool {
//!     // A fancy way to verify if the first character is uppercase
//!     name.chars().next().map(char::is_uppercase).unwrap_or(false)
//! }
//! ```
//!
//!
//! ## Feature flags
//!
//! * `serde1` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.
//!
//! ## Support Ukrainian military forces 🇺🇦
//!
//! Today I live in Berlin, I have the luxury to live a physically safe life.
//! But I am Ukrainian. The first 25 years of my life I spent in [Kharkiv](https://en.wikipedia.org/wiki/Kharkiv),
//! the second-largest city in Ukraine, 60km away from the border with russia. Today about [a third of my home city is destroyed](https://www.youtube.com/watch?v=ihoufBFSZds) by russians.
//! My parents, my relatives and my friends had to survive the artillery and air attack, living for over a month in basements.
//!
//! Some of them have managed to evacuate to EU. Some others are trying to live "normal lifes" in Kharkiv, doing there daily duties.
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
             sanitize(trim, lowercase)
             validate(not_empty)
         )]
        #[derive(*)]
        pub struct Email(String);

        let email = Email::new("  OH@my.example\n\n").unwrap();
        assert_eq!(email.into_inner(), "oh@my.example");

        assert_eq!(Email::new("  \n"), Err(EmailError::Missing));
    }

    #[test]
    fn test_amount_example() {
        #[nutype(validate(min = 100, max = 1_000))]
        #[derive(Debug, PartialEq, TryFrom)]
        pub struct Amount(u32);

        assert_eq!(Amount::try_from(99), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(1_001), Err(AmountError::TooBig));

        assert_eq!(Amount::try_from(100).unwrap().into_inner(), 100);
    }
}
