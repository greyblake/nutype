use nutype_derive::nutype;

#[nutype(
    sanitize(trim, lowercase)
    validate(present, min_len = 5, with = |e: &str| e.contains('@'))
)]
#[derive(Debug, FromStr, TryFrom, Borrow, PartialEq, Eq, Hash)]
pub struct Email(String);

/*
/// It should be cool here.
/// I hope that is fine.
///
/// # Example:
/// ```
/// let name = Name::from(" Anton\n");
/// assert_eq!(name.into_inner(), "Anton");
/// ```
#[nutype(validate(max = 100))]
#[derive(*)]
#[derive(Debug, Copy)]
pub struct Value(i32);
*/

fn main() {
    let email = Email::try_from("  example@MAIL.ORG ").unwrap();
    println!("\n\nemail = {:?}\n\n", email);
    assert_eq!(email.into_inner(), "example@mail.org");

    let my_mail: Email = "  THIS@cool.com".parse().unwrap();

    println!("my_mail = {:?}", my_mail);

    use std::collections::HashMap;

    let mut emails: HashMap<Email, u32> = HashMap::new();

    emails.insert(my_mail, 1);
    println!("{:?}", emails.get("this@cool.com"));

    // let value = Value::new(-15);
    // println!("value = {value:?}");
}
