use core::convert::TryFrom;
use nutype_derive::nutype;

// New crate name ideas:
// * newty (unfortunately taken :( )
// * typex
// * tipo
// * powertype
// * nutype
// * xtype

// #[nutype(
//     sanitize(trim, lowercase)
//     validate(present, min_len = 5)
// )]
// pub struct Email(String);

// TODO:
// * Implement validation for number
//   * Duplicates
//   * max cannot be smaller than min
//   * overlaps between clamp, min and max.
// * Support other numbers:
//   * Integers
//   * Floats

/// Just an age of the age.
#[nutype(
    sanitize(clamp(0, 200))
    validate(min = 0, max = 20)
)]
pub struct Age(i32);

fn main() {
    // let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    // println!("\n\nemail = {:?}\n\n", email);

    let age = Age::try_from(15).unwrap();
    println!("age = {age:?}");
}
