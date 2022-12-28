# Nutype

The newtype with guarantees.


## Quick start

```rust
use nutype::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(present)
)]
#[derive(Debug)]
pub struct Username(String);

assert_eq!(Username::new("  ").unwrap_err(), UsernameError::Missing);
assert_eq!(Username::new("  FOObar\n").unwrap().into_inner(), "foobar");
```

## Inner types

The following three categories of the inner types are supported:

* `String`
* Integer: `u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`
* Float: `f32`, `f64`

An inner type category determines set of available sanitizers, validators and derivable traits.

## String

At the moment the string inner type supports only `String` (owned) type.

### String sanitizers

| Sanitizer   | Description                                                                         | Example                                         |
|-------------|-------------------------------------------------------------------------------------|-------------------------------------------------|
| `trim`      | Removes leading and trailing whitespaces                                            | `trim`                                          |
| `lowercase` | Converts the string to lowercase                                                    | `lowercase`                                     |
| `uppercase` | Converts the string to uppercase                                                    | `uppercase`                                     |
| `with`      | Custom sanitizer. A function or closure that receives `String` and returns `String` | `with = \|mut s: String\| { s.truncate(5); s }` |

### String validators

| Validator | Description                                                                     | Error variant | Example                              |
|-----------|---------------------------------------------------------------------------------|---------------|--------------------------------------|
| `max_len` | Max length of the string                                                        | `TooLong`     | `max_len = 255`                      |
| `min_len` | Min length of the string                                                        | `TooShort`    | `min_len = 5`                        |
| `present` | Rejects an empty string                                                         | `Missing`     | `present`                            |
| `with`    | Custom validator. A function or closure that receives `&str` and returns `bool` | `Invalid`     | `with = \|s: &str\| s.contains('@')` |

### String derivable traits

The following traits can be derived for a string-based type:
`Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `From`, `TryFrom`, `Into`, `Hash`, `Borrow`, `Display`, `Serialize`, `Deserialize`.


## Integer

The integer inner types are: `u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`.

### Integer sanitizers

| Sanitizer | Description       | Example                            |
|-----------|-------------------|------------------------------------|
| `with`    | Custom sanitizer. | `with = \|raw\| raw.clamp(0, 100)` |

### Integer validators

| Validator | Description         | Error variant | Example                       |
|-----------|---------------------|---------------|-------------------------------|
| `max`     | Maximum valid value | `TooBig`      | `max = 99`                    |
| `min`     | Minimum valid value | `TooSmall`    | `min = 18`                    |
| `with`    | Custom validator    | `Invalid`     | `with = \|num\| num % 2 == 0` |

### Integer derivable traits

The following traits can be derived for an integer-based type:
`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Serialize`, `Deserialize`.


## Float

The float inner types are: `f32`, `f64`.

### Float sanitizers

| Sanitizer | Description       | Example                                |
|-----------|-------------------|----------------------------------------|
| `with`    | Custom sanitizer. | `with = \|val\| val.clamp(0.0, 100.0)` |

### Float validators

| Validator | Description         | Error variant | Example                       |
|-----------|---------------------|---------------|-------------------------------|
| `max`     | Maximum valid value | `TooBig`      | `max = 100.0`                 |
| `min`     | Minimum valid value | `TooSmall`    | `min = 0.0`                   |
| `with`    | Custom validator    | `Invalid`     | `with = \|val\| val != 50.0`  |

### Float derivable traits

The following traits can be derived for a float-based type:
`Debug`, `Clone`, `Copy`, `PartialEq`, `PartialOrd`, `FromStr`, `AsRef`, `Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Serialize`, `Deserialize`.


## Feature flags

* `serde1` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.

## License

TODO
