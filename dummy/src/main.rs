use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;

static PHONE_REGEX_ONCE_CELL: Lazy<Regex> = Lazy::new(|| Regex::new("[0-9]{3}-[0-9]{3}$").unwrap());

// #[nutype(
//     validate(regex = "foo")
// )]
#[nutype(validate(regex = self::PHONE_REGEX_ONCE_CELL))]
#[derive(Debug)]
pub struct Name(String);

fn main() {}
