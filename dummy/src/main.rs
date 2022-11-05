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

#[nutype(validate(min = 18, max = 99))]
pub struct Age(i32);

fn main() {
    // let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    // println!("\n\nemail = {:?}\n\n", email);
}
