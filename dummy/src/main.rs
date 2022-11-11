// TODO:
// * `derive(*)` - syntax to derive all possible traits
// * Custom sanitizers
//   * Strings (DONE)
//   * Numbers - TODO
// * Custom validations
//   * Strings - TODO
//   * Numbers - TODO
// * Regex
// * Support serde
//   * Serialize
//   * Deserialize
// * Support Arbitrary
// * Support decimals libraries:
//   * https://crates.io/crates/rust_decimal
// * Support time libraries (e.g. chrono)
// * Impl  "did you mean" hints:
//   * E.g. unknown validation rule `min`. Did you mean `min_len`?
// * Finalize syntax!
// * Setup CI
// * Find a way to bypass documentation comments
// Refactor parsers
// String sanitizers:
//   * capitalize
//   * truncate
//   * Remove extra spaces

use core::convert::TryFrom;
use nutype_derive::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(present, min_len = 5)
)]
pub struct Email(String);


// #[nutype(sanitize = [trim, lowercase], derive(*))]
// pub struct Email(String);

/// Just an age of the age.
#[nutype(
    sanitize(clamp(-1000, -10))
    validate(min = -2000, max = -10)
)]
pub struct Value(i32);

fn main() {
    let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    println!("\n\nemail = {:?}\n\n", email);
    assert_eq!(email.into_inner(), "example@mail.org");

    let value = Value::try_from(-15).unwrap();
    println!("value = {value:?}");
}
