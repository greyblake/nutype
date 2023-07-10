use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;

static PHONE_REGEX_ONCE_CELL: Lazy<Regex> = Lazy::new(|| Regex::new("[0-9]{3}-[0-9]{3}$").unwrap());

#[nutype(
    validate(regex = self::PHONE_REGEX_ONCE_CELL),
    default = "123-456",
)]
#[derive(Debug, Default)]
pub struct Name(String);

fn main() {
    let name = Name::default();
    println!("name = {name:?}");
}
