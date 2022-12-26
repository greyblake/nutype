# Nutype

Newtype with guarantees.


## Quick start

```rust
#[nutype(
    sanitize(trim, lowercase)
    validate(present)
)]
#[derive(Debug)]
pub struct Username(String);

assert_eq!(Username::new("  ").unwrap_err(), UsernameError::Missing);
assert_eq!(Username::new("  FOObar\n").unwrap().into_inner(), "foobar");
```


## Feature flags

* `serde1` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.

## License

TODO
