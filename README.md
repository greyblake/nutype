<p align="center"><img width="300" src="https://raw.githubusercontent.com/greyblake/nutype/master/art/rust_nutype.png" alt="Rust Nutype Logo"></p>
<h2 align="center">The newtype with guarantees.</h2>

<p align="center">
<a href="https://github.com/greyblake/nutype/actions/workflows/ci.yml" rel="nofollow"><img src="https://github.com/greyblake/nutype/actions/workflows/ci.yml/badge.svg" alt="Nutype Build Status"></a>
<a href="https://docs.rs/nutype" rel="nofollow"><img src="https://docs.rs/nutype/badge.svg" alt="Nutype Documentation"></a>
<p>

## Philosphy

Nutype embraces the simple idea: **the type system can be leveraged to track the fact that something was done, so there is no need to do it again**.

If a piece of data was once sanitized and validated we can rely on the types instead of sanitizing and validating again and again.


## Quick start

```rust
use nutype::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(present, max_len = 20)
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
    Err(UsernameError::Missing),
);

assert_eq!(
    Username::new("TheUserNameIsVeryVeryLong"),
    Err(UsernameError::TooLong),
);
```

Note, that we also explicitly got `UsernameError` enum generated.

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

You can derive traits. A lot of traits! For example:

```rust
#[nutype]
#[derive(*)]
struct Username(String);
```

The code above derives the following traits for `Username`: `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Hash`.
`*` is just a syntax sugar for "derive whatever makes sense to derive by default", which is very subjective and opinionated. It's rather an experimental feature that was born
from the fact that `#[nutype]` has to mess with `#[derive]` anyway, because users are not supposed to be able to derive traits like `DerefMut` or `BorrowMut`.
That would allow to mutate the inner (protected) value which undermines the entire idea of nutype.


## Inner types

Available sanitizers, validators and derivable traits are determined by the inner type, which falls into the following categories:
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

## Custom sanitizers

You can set custom sanitizers using option `with`.
A custom sanitizer is a function or closure that receives a value of an inner type with ownership and returns a sanitized value back.

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


## Feature flags

* `serde1` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.

## When nutype is a good fit for you?

* If you enjoy [newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)
  pattern and you like the idea of leveraging Rust type system to enforce correctness of the business logic.
* If you're a DDD fan, nutype is a great helper to make your domain models even more expressive.
* You want to prototype quickly without sacrificing quality.

## When nutype is not that good?

* You care too much about compiler time (nutype relies on heavy usage of proc macros).
* You think metaprogramming is too much implicit magic.
* IDEs may not be very helpful at giving you hints about proc macros.
* Design of nutype may enforce you to run unnecessary validation (e.g. on loading data from DB), which may have a negative impact if you aim for an extreme performance.

## How it works?


The following snippet

```rust
#[nutype(
    sanitize(trim, lowercase)
    validate(present, max_len = 20)
)]
pub struct Username(String);
```

eventually is transformed into something similar to this:

```rust
// Everything is wrapped into module,
// so the internal tuple value of Username is private and cannot be directly manipulated.
mod __nutype_private_Username__ {
    pub struct Username(String);

    pub enum UsernameError {
        // Occurres when a string is not present
        Missing,

        // Occurres when a string is longer than 255 chars.
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
                Err(UsernameError::Missing)
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

The `Username::into_inner(self)` allows to convert `Username` back into the inner type (`String`).

And of course the variants of `UsernameError` are derived from the validation rules.

**But the whole point of the `nutype` crate is that there is no legit way to obtain an instance of `Username` that violates the sanitization or validation rules.**
The author put a lot of effort into this. If you find a way to obtain the instance of a newtype bypassing the validation rules, please open an issue.

## A note about #[derive(...)]

You've got to know that the `#[nutype]` macro intercepts `#[derive(...)]` macro.
It's done on purpose to ensure that anything like `DerefMut` or `BorrowMut`, that can lead to violation of the validation rules is excluded.
The library takes a conservative approach and it has its downside: deriving traits which are not known to the library is not possible.

## Roadmap

* [ ] refactor the parser logic
* [ ] friendlier error messages:
  * [ ] `did you mean ...?` hints
  * [ ] intercept and explain why `DerefMut` and co cannot be derived
* [ ] for floats: add `finite` validator and allow to derive `Eq` and `Ord`
* [ ] integration with [diesel](https://github.com/diesel-rs/diesel)
* [ ] integration with [sqlx](https://github.com/launchbadge/sqlx)
* [ ] integration with [envconfig](https://github.com/greyblake/envconfig-rs)
* [ ] integration with [arbitrary](https://github.com/rust-fuzz/arbitrary)
* [ ] support `regex` to validate string types

## Support Ukrainian military forces 🇺🇦

Today I live in Berlin, I have a luxury to live a physically safe life.
But I am Ukrainian. The first 25 years of my life I spent in [Kharkiv](https://en.wikipedia.org/wiki/Kharkiv),
the second-largest city in Ukraine, 60km away from the border with russia. Today about [a third of my home city is destroyed](https://www.youtube.com/watch?v=ihoufBFSZds) by russians.
My parents, my relatives and my friends had to survive the artillery and air attack, living for over a month in basements.

Some of them have managed to evacuate to EU. Some others are trying to live "normal lifes" in Kharkiv, doing there daily duties.
And there are some who are at the front line right now, risking their lives every second to protect the rest.

I encourage you to donate to [Charity foundation of Serhiy Prytula](https://prytulafoundation.org/en).
Just pick the project you like and donate. This is one of the best known foundations, you can watch a [little documentary](https://www.youtube.com/watch?v=VlmWqoeub1Q) about it.
Your contribution to the Ukrainian military force is a contribution to my calmness, so I can spend more time developing the project.

Thank you.

## Similar projects

* [bounded-integer](https://github.com/Kestrer/bounded-integer) - Bounded integers for Rust.
* [refinement](https://docs.rs/refinement/latest/refinement/) - Convenient creation of type-safe refinement types (based on generics).
* [semval](https://github.com/slowtec/semval) - Semantic validation for Rust.
* [validator](https://github.com/Keats/validator) - Simple validation for Rust structs (powered by macros).



## License

MIT © Sergey Potapov
