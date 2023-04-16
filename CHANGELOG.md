### v0.3.0 - 2023-??-??
* Add `finite` validation for float types which checks against NaN and infinity.

### v0.2.0 - 2023-04-13

* [BREAKING] Rename string validator `present` -> `not_empty`. Rename error variant `Missing` -> `Empty`.
* [BREAKING] Rename feature `serde1` to `serde`.
* Introduce `new_unchecked` feature flag, that allows to bypass sanitization and validation.
* Support derive of `JsonSchema` of `schemars` crate (requires `schemars08` feature).
* Support string validation with `regex` (requires `regex` feature).

### v0.1.1 - 2023-02-11
* Initial release
