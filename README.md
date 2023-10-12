<p align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./art/rust_nutype_inverted.png">
  <source media="(prefers-color-scheme: light)" srcset="./art/rust_nutype.png">

  <img width="300" alt="Rust Nutype Logo" src="./art/rust_nutype.png">
</picture>
</p>
<h2 align="center">The newtype with guarantees.</h2>

<p align="center">
<a href="https://github.com/greyblake/nutype/actions/workflows/ci.yml" rel="nofollow"><img src="https://github.com/greyblake/nutype/actions/workflows/ci.yml/badge.svg" alt="Nutype Build Status"></a>
<a href="https://docs.rs/nutype" rel="nofollow"><img src="https://docs.rs/nutype/badge.svg" alt="Nutype Documentation"></a>
<a href="https://github.com/greyblake/nutype/discussions"><img src="https://img.shields.io/github/discussions/greyblake/nutype"/></a>
<p>

## Philosophy

Nutype embraces the simple idea: **the type system can be leveraged to track the fact that something was done, so there is no need to do it again**.

If a piece of data was once sanitized and validated we can rely on the types instead of sanitizing and validating again and again when we're in doubt.


## Quick start

```rust
use nutype::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(not_empty, max_len = 20)
)]
pub struct Username(String);
```

Now we can create usernames:

```rust
assert_eq!(
    Username::new("   FooBar  ").unwrap().into_inner(),
    "foobar"
);
```

But we cannot create invalid ones:

```rust
assert_eq!(
    Username::new("   "),
    Err(UsernameError::Empty),
);

assert_eq!(
    Username::new("TheUserNameIsVeryVeryLong"),
    Err(UsernameError::TooLong),
);
```

Note, that we also got `UsernameError` enum generated implicitly.

Ok, but let's try to obtain an instance of `Username` that violates the validation rules:

```rust
let username = Username("".to_string())

// error[E0423]: cannot initialize a tuple struct which contains private fields
```

```rust
let mut username = Username::new("foo").unwrap();
username.0 = "".to_string();

// error[E0616]: field `0` of struct `Username` is private
```

Haha. It's does not seem to be easy!


## A few more examples

Here are some other examples of what you can do with `nutype`.

You can skip `sanitize` and use a custom validator `with`:

```rust
#[nutype(validate(with = |n| n % 2 == 1))]
struct OddNumber(i64);
```

You can skip validation, if you need sanitization only:

```rust
#[nutype(sanitize(trim, lowercase))]
struct Username(String);
```

In that case `Username::new(String)` simply returns `Username`, not `Result`.

## Inner types

Available sanitizers, validators, and derivable traits are determined by the inner type, which falls into the following categories:
* String
* Integer (`u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`)
* Float (`f32`, `f64`)

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

| Validator   | Description                                                                     | Error variant   | Example                                      |
|-------------|---------------------------------------------------------------------------------|-----------------|----------------------------------------------|
| `max_len`   | Max length of the string (in chars, not bytes)                                  | `TooLong`       | `max_len = 255`                              |
| `min_len`   | Min length of the string (in chars, not bytes)                                  | `TooShort`      | `min_len = 5`                                |
| `not_empty` | Rejects an empty string                                                         | `Empty`         | `not_empty`                                  |
| `regex`     | Validates format with a regex. Requires `regex` feature.                        | `RegexMismatch` | `regex = "^[0-9]{7}$"` or `regex = ID_REGEX` |
| `with`      | Custom validator. A function or closure that receives `&str` and returns `bool` | `Invalid`       | `with = \|s: &str\| s.contains('@')`         |


#### Regex validation

Requirements:
* `regex` feature of `nutype` is enabled.
* You have to explicitly include `regex` and `lazy_static` as dependencies.

There are a number of ways you can use regex.

A regular expression can be defined right in place:

```rs
#[nutype(validate(regex = "^[0-9]{3}-[0-9]{3}$"))]
pub struct PhoneNumber(String);
```

or it can be defined with `lazy_static`:

```rs
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PHONE_NUMBER_REGEX: Regex = Regex::new("^[0-9]{3}-[0-9]{3}$").unwrap();
}

#[nutype(validate(regex = PHONE_NUMBER_REGEX))]
pub struct PhoneNumber(String);
```

or `once_cell`:

```rs
use once_cell::sync::Lazy;
use regex::Regex;

static PHONE_NUMBER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("[0-9]{3}-[0-9]{3}$").unwrap());

#[nutype(validate(regex = PHONE_NUMBER_REGEX))]
pub struct PhoneNumber(String);
```


### String derivable traits

The following traits can be derived for a string-based type:
`Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
`From`, `TryFrom`, `Into`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.


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
`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
`Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.


## Float

The float inner types are: `f32`, `f64`.

### Float sanitizers

| Sanitizer | Description       | Example                                |
|-----------|-------------------|----------------------------------------|
| `with`    | Custom sanitizer. | `with = \|val\| val.clamp(0.0, 100.0)` |

### Float validators

| Validator | Description                    | Error variant | Example                      |
|-----------|--------------------------------|---------------|------------------------------|
| `max`     | Maximum valid value            | `TooBig`      | `max = 100.0`                |
| `min`     | Minimum valid value            | `TooSmall`    | `min = 0.0`                  |
| `finite`  | Check against NaN and infinity | `NotFinite`   | `finite`                     |
| `with`    | Custom validator               | `Invalid`     | `with = \|val\| val != 50.0` |

### Float derivable traits

The following traits can be derived for a float-based type:
`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
`Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.

It's also possible to derive `Eq` and `Ord` if the validation rules guarantee that `NaN` is excluded.
This can be done applying by `finite` validation. For example:

```rust
#[nutype(validate(finite))]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Size(f64);
```

## Custom sanitizers

You can set custom sanitizers using the `with` option.
A custom sanitizer is a function or closure that receives a value of an inner type with ownership and returns a sanitized value.

For example, this one

```rust
#[nutype(sanitize(with = new_to_old))]
pub struct CityName(String);

fn new_to_old(s: String) -> String {
    s.replace("New", "Old")
}
```

is equal to the following one:

```rust
#[nutype(sanitize(with = |s| s.replace("New", "Old") ))]
pub struct CityName(String);
```

And works the same way:

```rust
let city = CityName::new("New York");
assert_eq!(city.into_inner(), "Old York");
```

## Custom validators

In similar fashion it's possible to define custom validators, but a validation function receives a reference and returns `bool`.
Think of it as a predicate.

```rust
#[nutype(validate(with = is_valid_name))]
pub struct Name(String);

fn is_valid_name(name: &str) -> bool {
    // A fancy way to verify if the first character is uppercase
    name.chars().next().map(char::is_uppercase).unwrap_or(false)
}
```

## Deriving recipes

### Deriving `Default`

```rs
#[nutype(default = "Anonymous")]
#[derive(Default)]
pub struct Name(String);
```

### Deriving `Eq` and `Ord` on float types

With nutype it's possible to derive `Eq` and `Ord` if there is `finite` validation set.
The `finite` validation ensures that the valid value excludes `NaN`.

```rs
#[nutype(validate(finite))]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Weight(f64);
```


## How to break the constraints?

First you need to know, you SHOULD NOT do it.

But let's pretend for some imaginary performance reasons you really need to avoid validation when instantiating a value of newtype
(e.g. loading earlier "validated" data from DB).

You can achieve this by enabling `new_unchecked` crate feature and marking a type with `new_unchecked`:

```rs
#[nutype(
    new_unchecked
    sanitize(trim)
    validate(min_len = 8)
)]
pub struct Name(String);

// Yes, you're forced to use `unsafe` here, so everyone will point fingers at YOU.
let name = unsafe { Name::new_unchecked(" boo ".to_string()) };

// `name` violates the sanitization and validation rules!!!
assert_eq!(name.into_inner(), " boo ");
```

## Feature flags

* `serde` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.
* `regex` - allows to use `regex = ` validation on string-based types. Note: your crate also has to explicitly have `regex` and `lazy_static` within dependencies.
* `schemars08` - allows to derive [`JsonSchema`](https://docs.rs/schemars/0.8.12/schemars/trait.JsonSchema.html) trait of [schemars](https://crates.io/crates/schemars) crate. Note that at the moment validation rules are not respected.
* `new_unchecked` - enables generation of unsafe `::new_unchecked()` function.

## When nutype is a good fit for you?

* If you enjoy [newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)
  pattern and you like the idea of leveraging the Rust type system to enforce the correctness of the business logic.
* If you're a DDD fan, nutype is a great helper to make your domain models even more expressive.
* You want to prototype quickly without sacrificing quality.

## When nutype is not that good?

* You care too much about compiler time (nutype relies on heavy usage of proc macros).
* You think metaprogramming is too much implicit magic.
* IDEs may not be very helpful at giving you hints about proc macros.
* Design of nutype may enforce you to run unnecessary validation (e.g. on loading data from DB), which may have a negative impact if you aim for extreme performance.

## How it works?


The following snippet

```rust
#[nutype(
    sanitize(trim, lowercase)
    validate(not_empty, max_len = 20)
)]
pub struct Username(String);
```

eventually is transformed into something similar to this:

```rust
// Everything is wrapped into the module,
// so the internal tuple value of Username is private and cannot be directly manipulated.
mod __nutype_private_Username__ {
    pub struct Username(String);

    pub enum UsernameError {
        // Occurs when a string is empty
        Empty,

        // Occurs when a string is longer than 255 chars.
        TooLong,
    }

    impl Username {
        // The only legit way to construct Username.
        // All other constructors (From, FromStr, Deserialize, etc.)
        // are built on top of this one.
        pub fn new(raw_username: impl Into<String>) -> Result<Username, UsernameError> {
            // Sanitize
            let sanitized_username = raw_username.into().trim().lowercase();

            // Validate
            if sanitized_username.empty() {
                Err(UsernameError::Empty)
            } else if (sanitized_username.len() > 40 {
                Err(UsernameError::TooLong)
            } else {
                Ok(Username(sanitized_username))
            }
        }

        // Convert back to the inner type.
        pub fn into_inner(self) -> String {
            self.0
        }
    }
}

pub use __nutype_private_Username__::{Username, UsernameError};
```

As you can see, `#[nutype]`  macro gets sanitization and validation rules and turns them into Rust code.

The `Username::new()` constructor performs sanitization and validation and in case of success returns an instance of `Username`.

The `Username::into_inner(self)` allows converting `Username` back into the inner type (`String`).

And of course, the variants of `UsernameError` are derived from the validation rules.

**But the whole point of the `nutype` crate is that there is no legit way to obtain an instance of `Username` that violates the sanitization or validation rules.**
The author put a lot of effort into this. If you find a way to obtain the instance of a newtype bypassing the validation rules, please open an issue.

## A note about #[derive(...)]

You've got to know that the `#[nutype]` macro intercepts `#[derive(...)]` macro.
It's done on purpose to ensure that anything like `DerefMut` or `BorrowMut`, that can lead to a violation of the validation rules is excluded.
The library takes a conservative approach and it has its downside: deriving traits that are not known to the library is not possible.

## Support Ukrainian military forces 🇺🇦

Today I live in Berlin, I have the luxury to live a physically safe life.
But I am Ukrainian. The first 25 years of my life I spent in [Kharkiv](https://en.wikipedia.org/wiki/Kharkiv),
the second-largest city in Ukraine, 60km away from the border with russia. Today about [a third of my home city is destroyed](https://www.youtube.com/watch?v=ihoufBFSZds) by russians.
My parents, my relatives and my friends had to survive the artillery and air attack, living for over a month in basements.

Some of them have managed to evacuate to EU. Some others are trying to live "normal lifes" in Kharkiv, doing there daily duties.
And some are at the front line right now, risking their lives every second to protect the rest.

I encourage you to donate to [Charity foundation of Serhiy Prytula](https://prytulafoundation.org/en).
Just pick the project you like and donate. This is one of the best-known foundations, you can watch a [little documentary](https://www.youtube.com/watch?v=VlmWqoeub1Q) about it.
Your contribution to the Ukrainian military force is a contribution to my calmness, so I can spend more time developing the project.

Thank you.

## Similar projects

* [prae](https://github.com/teenjuna/prae) - A very similar crate that aims to solve the same problems but with slightly different approach.
* [bounded-integer](https://github.com/Kestrer/bounded-integer) - Bounded integers for Rust.
* [refinement](https://docs.rs/refinement/latest/refinement/) - Convenient creation of type-safe refinement types (based on generics).
* [semval](https://github.com/slowtec/semval) - Semantic validation for Rust.
* [validator](https://github.com/Keats/validator) - Simple validation for Rust structs (powered by macros).

## License

MIT © [Serhii Potapov](https://www.greyblake.com)
