<p align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/greyblake/nutype/master/art/rust_nutype_inverted.png">
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/greyblake/nutype/master/art/rust_nutype.png">

  <img width="300" alt="Rust Nutype Logo" src="https://raw.githubusercontent.com/greyblake/nutype/master/art/rust_nutype.png">
</picture>
</p>
<h2 align="center">The newtype with guarantees.</h2>

<p align="center">
<a href="https://github.com/greyblake/nutype/actions/workflows/ci.yml" rel="nofollow"><img src="https://github.com/greyblake/nutype/actions/workflows/ci.yml/badge.svg" alt="Nutype Build Status"></a>
<a href="https://docs.rs/nutype" rel="nofollow"><img src="https://docs.rs/nutype/badge.svg" alt="Nutype Documentation"></a>
<a href="https://github.com/greyblake/nutype/discussions"><img src="https://img.shields.io/github/discussions/greyblake/nutype"/></a>
<p>


Nutype is a proc macro that allows adding extra constraints like _sanitization_ and _validation_ to the regular [newtype pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html). The generated code makes it impossible to instantiate a value without passing the checks. It works this way even with `serde` deserialization.


* [Quick start](#quick-start)
* [Inner types](#inner-types) ([String](#string) | [Integer](#integer) | [Float](#float) | [Other](#other-inner-types))
* [Custom](#custom-sanitizers) ([sanitizers](#custom-sanitizers) | [validators](#custom-validators))
* [Recipes](#recipes)
* [Breaking constraints with new_unchecked](#breaking-constraints-with-new_unchecked)
* [Feature Flags](#feature-flags)
* [Support Ukrainian military forces](#support-ukrainian-military-forces)
* [Similar projects](#similar-projects)

## Quick start

```rust
use nutype::nutype;

// Define newtype Username
#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_max = 20),
    derive(Debug, PartialEq, Clone),
)]
pub struct Username(String);

// We can obtain a value of Username with `::new()`.
// Note that Username holds a sanitized string
assert_eq!(
    Username::new("   FooBar  ").unwrap().into_inner(),
    "foobar"
);

// It's impossible to obtain an invalid Username
// Note that we also got `UsernameError` enum generated implicitly
// based on the validation rules.
assert_eq!(
    Username::new("   "),
    Err(UsernameError::NotEmptyViolated),
);
assert_eq!(
    Username::new("TheUserNameIsVeryVeryLong"),
    Err(UsernameError::LenCharMaxViolated),
);
```

For more please see:
* [Examples](https://github.com/greyblake/nutype/tree/master/examples)
* [Tests](https://github.com/greyblake/nutype/tree/master/test_suite/tests)


## Inner types

Available sanitizers, validators, and derivable traits are determined by the inner type, which falls into the following categories:
* String
* Integer (`u8`, `u16`,`u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`)
* Float (`f32`, `f64`)
* Anything else

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

| Validator      | Description                                                                     | Error variant        | Example                                      |
|----------------|---------------------------------------------------------------------------------|----------------------|----------------------------------------------|
| `len_char_min` | Min length of the string (in chars, not bytes)                                  | `LenCharMinViolated` | `len_char_min = 5`                           |
| `len_char_max` | Max length of the string (in chars, not bytes)                                  | `LenCharMaxViolated` | `len_char_max = 255`                         |
| `not_empty`    | Rejects an empty string                                                         | `NotEmptyViolated`   | `not_empty`                                  |
| `regex`        | Validates format with a regex. Requires `regex` feature.                        | `RegexViolated`      | `regex = "^[0-9]{7}$"` or `regex = ID_REGEX` |
| `predicate`    | Custom validator. A function or closure that receives `&str` and returns `bool` | `PredicateViolated`  | `predicate = \|s: &str\| s.contains('@')`    |


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

| Validator           | Description           | Error variant             | Example                              |
| ------------------- | --------------------- | ------------------------- | ------------------------------------ |
| `less`              | Exclusive upper bound | `LessViolated`            | `less = 100`                         |
| `less_or_equal`     | Inclusive upper bound | `LessOrEqualViolated`     | `less_or_equal = 99`                 |
| `greater`           | Exclusive lower bound | `GreaterViolated`         | `greater = 17`                       |
| `greater_or_equal`  | Inclusive lower bound | `GreaterOrEqualViolated`  | `greater_or_equal = 18`              |
| `predicate`         | Custom predicate      | `PredicateViolated`       | `predicate = \|num\| num % 2 == 0`   |

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

| Validator          | Description                      | Error variant            | Example                             |
| ------------------ | -------------------------------- | ---------------------    | ----------------------------------- |
| `less`             | Exclusive upper bound            | `LessViolated`           | `less = 100.0`                      |
| `less_or_equal`    | Inclusive upper bound            | `LessOrEqualViolated`    | `less_or_equal = 100.0`             |
| `greater`          | Exclusive lower bound            | `GreaterViolated`        | `greater = 0.0`                     |
| `greater_or_equal` | Inclusive lower bound            | `GreaterOrEqualViolated` | `greater_or_equal = 0.0`            |
| `finite`           | Check against NaN and infinity   | `FiniteViolated`         | `finite`                            |
| `predicate`        | Custom predicate                 | `PredicateViolated`      | `predicate = \|val\| val != 50.0`   |

### Float derivable traits

The following traits can be derived for a float-based type:
`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromStr`, `AsRef`, `Deref`,
`Into`, `From`, `TryFrom`, `Hash`, `Borrow`, `Display`, `Default`, `Serialize`, `Deserialize`.

It's also possible to derive `Eq` and `Ord` if the validation rules guarantee that `NaN` is excluded.
This can be done applying by `finite` validation. For example:

```rust
#[nutype(
    validate(finite),
    derive(PartialEq, Eq, PartialOrd, Ord),
)]
struct Size(f64);
```

## Other inner types

For any other type it is possible to define custom sanitizers with `with` and custom
validations with `predicate`:

```rs
use nutype::nutype;

#[nutype(
    derive(Debug, PartialEq, Deref, AsRef),
    sanitize(with = |mut guests| { guests.sort(); guests }),
    validate(predicate = |guests| !guests.is_empty() ),
)]
pub struct GuestList(Vec<String>);

// Empty list is not allowed
assert_eq!(
    GuestList::new(vec![]),
    Err(GuestListError::PredicateViolated)
);

// Create the list of our guests
let guest_list = GuestList::new(vec![
    "Seneca".to_string(),
    "Marcus Aurelius".to_string(),
    "Socrates".to_string(),
    "Epictetus".to_string(),
]).unwrap();

// The list is sorted (thanks to sanitize)
assert_eq!(
    guest_list.as_ref(),
    &[
        "Epictetus".to_string(),
        "Marcus Aurelius".to_string(),
        "Seneca".to_string(),
        "Socrates".to_string(),
    ]
);

// Since GuestList derives Deref, we can use methods from `Vec<T>`
// due to deref coercion (if it's a good idea or not, it's left up to you to decide!).
assert_eq!(guest_list.len(), 4);

for guest in guest_list.iter() {
    println!("{guest}");
}

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
#[nutype(validate(predicate = is_valid_name))]
pub struct Name(String);

fn is_valid_name(name: &str) -> bool {
    // A fancy way to verify if the first character is uppercase
    name.chars().next().map(char::is_uppercase).unwrap_or(false)
}
```

## Recipes

### Derive `Default`

```rs
#[nutype(
    derive(Default),
    default = "Anonymous",
)]
pub struct Name(String);
```

### Derive `Eq` and `Ord` on float types

With nutype it's possible to derive `Eq` and `Ord` if there is `finite` validation set.
The `finite` validation ensures that the valid value excludes `NaN`.

```rs
#[nutype(
    validate(finite),
    derive(PartialEq, Eq, PartialOrd, Ord),
)]
pub struct Weight(f64);
```


## Breaking constraints with new_unchecked

It's discouraged, but it's possible to bypass the constraints by enabling `new_unchecked` crate feature and marking a type with `new_unchecked`:

```rs
#[nutype(
    new_unchecked,
    sanitize(trim),
    validate(len_char_min = 8)
)]
pub struct Name(String);

// Yes, you're forced to use `unsafe` here, so everyone will point fingers at YOU.
let name = unsafe { Name::new_unchecked(" boo ".to_string()) };

// `name` violates the sanitization and validation rules!!!
assert_eq!(name.into_inner(), " boo ");
```

## Feature flags

* `arbitrary` - enables derive of [`arbitrary::Arbitrary`](https://docs.rs/arbitrary/latest/arbitrary/trait.Arbitrary.html).
* `new_unchecked` - enables generation of unsafe `::new_unchecked()` function.
* `regex` - allows to use `regex = ` validation on string-based types. Note: your crate also has to explicitly have `regex` and `lazy_static` within dependencies.
* `serde` - integrations with [`serde`](https://crates.io/crates/serde) crate. Allows to derive `Serialize` and `Deserialize` traits.
* `schemars08` - allows to derive [`JsonSchema`](https://docs.rs/schemars/0.8.12/schemars/trait.JsonSchema.html) trait of [schemars](https://crates.io/crates/schemars) crate. Note that at the moment validation rules are not respected.
* `std` - enabled by default. Use `default-features = false` to disable.

## When nutype is a good fit for you?

* If you enjoy [newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)
  pattern and you like the idea of leveraging the Rust type system to enforce the correctness of the business logic.
* If you want to use type system to hold invariants
* If you're a DDD fan, nutype is a great helper to make your domain models even more expressive.
* You want to prototype quickly without sacrificing quality.

## When nutype is not that good?

* You care too much about compiler time (nutype relies on heavy usage of proc macros).
* You think metaprogramming is too much implicit magic.
* IDEs may not be very helpful at giving you hints about proc macros.
* Design of nutype may enforce you to run unnecessary validation (e.g. on loading data from DB), which may have a negative impact if you aim for extreme performance.

## A note about #[derive(...)]

You've got to know that the `#[nutype]` macro intercepts `#[derive(...)]` macro.
It's done on purpose to ensure that anything like `DerefMut` or `BorrowMut`, that can lead to a violation of the validation rules is excluded.
The library takes a conservative approach and it has its downside: deriving traits that are not known to the library is not possible.

## Support Ukrainian military forces

Today I live in Berlin, I have the luxury to live a physically safe life.
But I am Ukrainian. The first 25 years of my life I spent in [Kharkiv](https://en.wikipedia.org/wiki/Kharkiv),
the second-largest city in Ukraine, 60km away from the border with russia. Today about [a third of my home city is destroyed](https://www.youtube.com/watch?v=ihoufBFSZds) by russians.
My parents, my relatives and my friends had to survive the artillery and air attack, living for over a month in basements.

Some of them have managed to evacuate to EU. Some others are trying to live "normal lives" in Kharkiv, doing there daily duties.
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
