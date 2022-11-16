use nutype_derive::nutype;

// #[nutype(
//     sanitize(trim, lowercase)
//     validate(present, min_len = 5, with = |e: &str| e.contains('@'))
// )]
// pub struct Email(String);

// fn sanitize_value(val: i32) -> i32 {
//     if val > 100 {
//         100
//     } else {
//         val
//     }
// }

/// It should be cool here.
/// I hope that is fine.
///
/// # Example:
/// ```
/// let name = Name::from(" Anton\n");
/// assert_eq!(name.into_inner(), "Anton");
/// ```
#[nutype(validate(max = 100))]
#[derive(Debug, Copy)]
pub struct Value(i32);

fn main() {
    // let email = Email::try_from("  EXAMPLE@mail.ORG\n").unwrap();
    // println!("\n\nemail = {:?}\n\n", email);
    // assert_eq!(email.into_inner(), "example@mail.org");

    // let value = Value::from(-15);
    // println!("value = {value:?}");
}
