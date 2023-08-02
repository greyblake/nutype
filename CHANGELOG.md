### v0.4.0 - 2023-xx-xx
* [BREAKING] Removal of asterisk derive
* [BREAKING] Use commas to separate high level attributes
* [BREAKING] Traits are derived with `#[nutype(derive(Debug))]`. The regular `#[derive(Debug)]` syntax is not supported anymore.
* [BREAKING] Rename `with = ` validator to `predicate =`

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
