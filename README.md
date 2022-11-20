## Roadmap

### TODO:
* `derive(*)` - syntax to derive all possible traits
  * Alternatively allow the regular `#[derive(Debug, Copy)]` syntax. The attributes can be read from syn
* Impl FromStr for String types
* Rename inner generated `validate` and `sanitize` methods into something more unique. Otherwise it may conflict because of `use super::*;`
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
* Address TODOs
* Refactor parsers
* Number sanitizers:
  * Replace `clamp(a, b)` with something like `min = a, max = b`
* String sanitizers:
  * capitalize
  * truncate
  * Remove extra spaces
* Extra validations for floats:
  * `is_number` / `is_finite` (aka not NaN, and not `Inifinity`)
* Generate documentation automatically.


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
