
### v0.2.0 - 2023-??-??

* [BREAKING] Rename string validator `present` -> `not_empty`. Rename error variant `Missing` -> `Empty`.
* Introduce `new_unchecked` feature flag, that allows to bypass sanitization and validation.
* Support derive of `JsonSchema` of `schemars` crate (requires `schemars08` feature).

### v0.1.1 - 2023-02-11
* Initial release
