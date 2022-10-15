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
    validate(present, min_len = 6, max_len = 255)
    sanitize(trim, uppercase)
)]
pub struct Email(String);

fn main() {
    let raw_email = "\tBlake13@gmail.COM  \n";
    let email = Email::try_from(raw_email.to_string()).unwrap();
    println!("\n\nemail = {:?}\n\n", email);
}
