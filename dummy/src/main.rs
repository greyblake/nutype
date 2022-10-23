use core::convert::TryFrom;
use nutype_derive::nutype;

// New crate name ideas:
// * newty (unfortunately taken :( )
// * typex
// * tipo
// * powertype
// * nutype
// * xtype

#[nutype(
    sanitize(trim, lowercase)
    validate(present, min_len = 6)
)]
pub struct Email(String);

/*
#[derive(nutype::TryFrom)]
#[sanitize(trim, lowercase)]
#[validate(present, min_len=6, max_len=255)]
pub struct Email(String);
*/

fn main() {
    let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    println!("\n\nemail = {:?}\n\n", email);
}
