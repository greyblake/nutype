### v0.5.1 - 2024-xx-xx

* **[FEATURE]** In `no_std` generate implementation of `::core::error::Error` if Rust version is 1.81 or higher.
* **[FIX]** Enable to specify custom error as a path (see [#186](https://github.com/greyblake/nutype/issues/186), [#187](https://github.com/greyblake/nutype/pull/187))
* **[FIX]** Make `Deserialize` derive compile when combination of `no_std` and `serde` features are used ([#182](https://github.com/greyblake/nutype/issues/182))
* **[FIX]** Fix lint warnings about inner generated module

### v0.5.0 - 2024-09-02

- **[FEATURE]** Added support for custom error types and validation functions via the `error` and `with` attributes.
- **[BREAKING]** Replaced `lazy_static` with [`std::sync::LazyLock`](https://doc.rust-lang.org/stable/std/sync/struct.LazyLock.html) for regex validation. This requires Rust 1.80 or higher and may cause compilation issues on older Rust versions due to the use of `std::sync::LazyLock`. If upgrading Rust isn't an option, you can still use `lazy_static` explicitly as a workaround.
- **[BREAKING]** The fallible `::new()` constructor has been fully replaced by `::try_new()`.

### v0.4.3 - 2024-07-06

* Support generics
* [DEPRECATION] Fallible constructor `::new()` is deprecated. Users should use `::try_new()` instead.
* [FIX] Use absolute path for `::core::result::Result` when generating code for `derive(TryFrom)`.

### v0.4.2 - 2024-04-07

* Support `no_std` ( the dependency needs to be declared as `nutype = { default-features = false }` )
* Support integration with [`arbitrary`](https://crates.io/crates/arbitrary) crate (see `arbitrary` feature).
  * Support `Arbitrary` for integer types
  * Support `Arbitrary` for float types
  * Support `Arbitrary` for string inner types
  * Support `Arbitrary` for any inner types
* Possibility to specify boundaries (`greater`, `greater_or_equal`, `less`, `less_or_equal`, `len_char_min`, `len_char_max`) with expressions or named constants.
* Add `#[inline]` attribute to trivial functions
* Improve error messages

### v0.4.1 - 2024-04-07

* Failed release. Includes everything from v0.4.2 except support of `Arbitrary` for `String` based types.

### v0.4.0 - 2023-11-21
* Support of arbitrary inner types with custom sanitizers and validators.
* Add numeric validator `greater`
* Add numeric validator `less`
* [BREAKING] Removal of asterisk derive
* [BREAKING] Use commas to separate high level attributes
* [BREAKING] Traits are derived with `#[nutype(derive(Debug))]`. The regular `#[derive(Debug)]` syntax is not supported anymore.
* [BREAKING] Validator `with` has been renamed to `predicate` to reflect the boolean nature of its range
* [BREAKING] String validator `min_len` has been renamed to `len_char_min` to reflect that is based on UTF8 chars.
* [BREAKING] String validator `max_len` has been renamed to `len_char_max` to reflect that is based on UTF8 chars.
* [BREAKING] Rename numeric validator `max` to `less_or_equal`
* [BREAKING] Rename numeric validator `min` to `greater_or_equal`
* [BREAKING] Rename error variants to follow the following formula: `<ValidationRule>Violated`. This implies the following renames:
  * `TooShort` -> `LenCharMinViolated`
  * `TooLong` -> `LenCharMaxViolated`
  * `Empty` -> `NotEmptyViolated`
  * `RegexMismatch` -> `RegexViolated`
  * `Invalid` -> `PredicateViolated`
  * `TooBig` -> `LessOrEqualViolated`
  * `TooSmall` -> `GreaterOrEqualViolated`
  * `NotFinite` -> `FiniteViolated`
* Better error messages: in case of unknown attribute, validator or sanitizer the possible values are listed.
* [FIX] Make derived `Deserialize` work with RON format

### v0.3.1 - 2023-06-30
* Support deriving of `Deref`

### v0.3.0 - 2023-06-25
* [BREAKING] `min_len` and `max_len` validators run against number of characters in a string (`val.chars().count()`), not number of bytes (`val.len()`).
* Add `finite` validation for float types which checks against NaN and infinity.
* Support deriving of `Default`
* Support deriving of `Eq` and `Ord` on float types (if `finite` validation is present)
* Support deriving of `TryFrom` for types without validation (in this case Error type is `std::convert::Infallible`)

### v0.2.0 - 2023-04-13

* [BREAKING] Rename string validator `present` -> `not_empty`. Rename error variant `Missing` -> `Empty`.
* [BREAKING] Rename feature `serde1` to `serde`.
* Introduce `new_unchecked` feature flag, that allows to bypass sanitization and validation.
* Support derive of `JsonSchema` of `schemars` crate (requires `schemars08` feature).
* Support string validation with `regex` (requires `regex` feature).

### v0.1.1 - 2023-02-11
* Initial release
