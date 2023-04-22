## TODO
* Update to Rust 1.69

## Tech debt
* Make individual UI tests possible to opt-out depending on the feature flags.
* Use `enum-kind` crate

### TODO Refactor:
* Refactor parsers
* Try to generelize the parsing, validation and generation over types

### TODO UI:
* UI: Impl  "did you mean" hints:
  UI: * E.g. unknown validation rule `min`. Did you mean `min_len`?
* UI: Generate documentation automatically.
* UI: Intercept derive of DerefMut, AsMut, BorrowMut and print an explaining error message
* UI: On derive handle the following dependencies:
  * Copy requires Clone
  * Ord requires PartialOrd
  * Eq requires PartialEq

## Ideas for recipes (docs)
* Derive Eq and Ord on float based type if `finite` validation is set
* Validating strings with regex


### Later
* Support decimals libraries:
  * https://crates.io/crates/rust_decimal
* Support Arbitrary
* See https://github.com/CryptArchy/regex_generate to impl support with arbitrary
* Setup CI
* String sanitizers:
  * capitalize
  * truncate
  * Remove extra spaces
* Extra validations for floats:
  * `is_number` / `is_finite` (aka not NaN, and not `Inifinity`)
  * This should allow to derive Eq and Ord
* Consider extending errors to keep the invalid value?

### Maybe
* Add #[repr(transparent)]
* Add #[allow(non_snake_case)] for module names
* Support time libraries (e.g. chrono, time)


### Done
* Custom sanitizers for strings
* Custom validators for strings
* Custom sanitizers for numbers
* Custom validators for numbers
* Setup compiletests
* Use `new`, instead of `from` and `try_from`
* Respect visibility
* Respect documentation
* Implement std::error::Error for errors
* Support derive for String
* Support derive of From and TryFrom for String types
* Add UI tests to detect conflicts:
  * derive(TryFrom) without validations
  * derive(From) with validations
* Support derive Hash for String
* Impl FromStr for String types
* Support derive of Borrow<str> and Borrow<String> for String types
* Refactor numbers and split into groups: integer and float.
* Support derive for integers
* Support derive for floats
* Support derive of Into trait for String
* Support derive of Into trait for integers
* Support derive of Into trait for floats
* Refactor: extract common generator functions
* Cleanup tests: split number tests into integer and float
* Use absolute path to `Result` in the generated code
* Rename inner generated `validate` and `sanitize` methods into something more unique. Otherwise it may conflict because of `use super::*;`
* Rename default inner modules into something less scary
* Impl FromStr for floats
* Impl Display for errors on integers and floats. + add tests
* Improve Display for parse error of float: src/common/gen/parse_error.rs
* Impl FromStr for integer
* Derive Display
* Rename nutype_derive to nutype_macros
* Rename nutype_test_suite to `test_suite`
* Remove sanitizer `clamp(a, b)` from integer
* Remove sanitizer `clamp(a, b)` from float
* Address TODO, todo!(), unimplemented!(), etc.
* Reduce duplications: the types share mainly same structure
* UI: Hide private module docs: Use #[doc(hidden)] on the module!
* UI: Validate and show helpful error on attempt to make inner field public. E.g. `Value(pub i32)`
* Find a way to bypass serde feature flag from `nutype` to `nutype_macros`.
* Support serde: impl Serialize
* Impl Serialize tests for: integer, float, string
* Impl Clone tests for: integer, float, string
* Impl Copy tests for: integer, float
* Support serde: Deserialize
* Use `Infallible` type
* Rearrange parsing modules
* Address unwraps: replace with returning an error or expect()
* Rearrange models
* Use newtype for type name
* Run UI tests only against stable
* Add LICENSE
* Add the arts to the repo and add the logo to README
* Create a logo?
  * Use this font: https://www.fontspace.com/stoner-font-f81576
* Add a tiny philosophy section to README
* Add section with the similar projects
* Write about custom validators and sanitizers
* Add all Meta info to Cargo.toml and github repo
* Add the docs to lib.rs
* Add a CHANGELOG.md
* Publish to crates.io
* CI: github actions
* Write a blog article
* Fix typos in README and `lib.rs`
* new_unchecked: add the crate feature flag
* new_unchecked: add the check against the flag when parsing
* new_unchecked: add test coverage
* new_unchecked: add docs to README
* new_unchecked: add docs to lib.rs
* new_unchecked: review the PR: https://github.com/greyblake/nutype/pull/16
* Merge code examples in README and lib.rs into bigger chunks, so doc tests can run
* Reuse gen_impl_into_inner
* Refactor: Use newtype for errors (e.g. error_type_name, etc)
* JsonSchema: Add unit tests
* JsonSchema: UI test
* Validation with regex
* JsonSchema: add README, lib docs
* Implement unit tests for float Ord
  * Test cmp
  * Test sort()
* Implement UI tests for derive(Ord), cover cases:
  * When finite is not set
  * When Eq is not derived
  * When PartialOrd is not derived
* Prop-based tests for Ord
* Add note about Eq and Ord on floats in README
* Update CHANGELOG
