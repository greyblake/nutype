## Roadmap

### TODO:
* Respect visibility
* Respect documentation
* Use `new`, instead of `from` and `try_from`
* `derive(*)` - syntax to derive all possible traits
* Regex
* Support serde
  * Serialize
  * Deserialize
* Support Arbitrary
* Support decimals libraries:
  * https://crates.io/crates/rust_decimal
* Support time libraries (e.g. chrono)
* Impl  "did you mean" hints:
  * E.g. unknown validation rule `min`. Did you mean `min_len`?
* Finalize syntax!
* Setup CI
Refactor parsers
Number sanitizers:
  * Replace `clamp(a, b)` with something like `min = a, max = b`
String sanitizers:
  * capitalize
  * truncate
  * Remove extra spaces
* Impl FromStr for String types


### Done
* Custom sanitizers for strings
* Custom validators for strings
* Custom sanitizers for numbers
* Custom validators for numbers
* Setup compiletests
