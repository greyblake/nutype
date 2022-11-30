## Roadmap

### TODO:
* Support derive of Borrow<str> and Borrow<String> for String types
* Support derive for numbers
* Rename inner generated `validate` and `sanitize` methods into something more unique. Otherwise it may conflict because of `use super::*;`
* Support serde
  * Serialize
  * Deserialize
* Support Arbitrary
* Support decimals libraries:
  * https://crates.io/crates/rust_decimal
* Regex
  * See https://github.com/CryptArchy/regex_generate to impl support with arbitrary
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
  * This should allow to derive Eq and Ord
* Generate documentation automatically.
* Intercept derive of DerefMut, AsMut, BorrowMut and print an explaining error message


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



## Similar crates

* bounded_integer
* semval
* refinement
