//! Examples of nutype around references to DSTs (`&'a str`, `&'a [T]`).
//!
//! `str` and `[T]` are dynamically sized types, so they can only appear behind
//! a pointer such as `&'a str` or `&'a [T]`. nutype supports these as inner
//! types since v0.6.0 (`&'a str`) and via generics for slice references.

use nutype::nutype;

// A non-empty trimmed name borrowed from somewhere else.
#[nutype(
    validate(predicate = |s| !s.trim().is_empty()),
    derive(Debug, Clone, Copy, PartialEq, AsRef),
)]
pub struct Name<'a>(&'a str);

// A non-empty slice of any T, useful for read-only views of validated data.
#[nutype(
    validate(predicate = |s| !s.is_empty()),
    derive(Debug, Clone, Copy, PartialEq),
)]
pub struct NonEmptySlice<'a, T>(&'a [T]);

fn main() {
    // &'a str: borrowed string with a non-empty invariant.
    let raw = String::from("  Alice  ");
    let name = Name::try_new(&raw).unwrap();
    assert_eq!(name.as_ref(), &"  Alice  ");

    // Whitespace-only input is rejected.
    assert!(Name::try_new("   ").is_err());

    // A &'static str works just as well.
    let constant: Name<'static> = Name::try_new("Bob").unwrap();
    assert_eq!(constant.as_ref(), &"Bob");

    // &'a [T]: borrowed slice that must be non-empty.
    let numbers = vec![1, 2, 3];
    let slice = NonEmptySlice::try_new(numbers.as_slice()).unwrap();
    assert_eq!(slice.into_inner(), &[1, 2, 3]);

    // An empty slice is rejected.
    let empty: &[i32] = &[];
    assert!(NonEmptySlice::try_new(empty).is_err());

    println!("str_reference example: all assertions passed");
}
