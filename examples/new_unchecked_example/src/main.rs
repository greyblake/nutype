//! This example illustrates usage of `new_unchecked` feature that allows to create a value of
//! the type, bypassing the sanitization and validation checks.
//!
//! Use it carefully.

use nutype::nutype;

#[nutype(new_unchecked, sanitize(trim), validate(not_empty))]
pub struct Name(String);

fn main() {
    // Yes, you're forced to use `unsafe` here, so everyone will point fingers at YOU.
    let name = unsafe { Name::new_unchecked(" ".to_string()) };

    // `name` violates the sanitization and validation rules!!!
    assert_eq!(name.into_inner(), " ");
}
