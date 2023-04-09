use nutype::nutype;
use schemars::JsonSchema;
#[nutype(validate(max = 12.34))]
#[derive(FromStr, Display, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct Dist(f64);

#[nutype(
    new_unchecked
    validate(min = 18, max = 99)
)]
#[derive(FromStr, Display, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct Age(u8);

#[nutype(new_unchecked)]
#[derive(Debug, FromStr, Display, Clone, Serialize, JsonSchema)]
pub struct Username(String);

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PIN_CODE_REGEX_LAZY_STATIC: Regex = Regex::new("^[0-9]{4}$").unwrap();
}

#[nutype(validate(regex = PIN_CODE_REGEX_LAZY_STATIC))]
// #[nutype(validate(regex = "^[0-9]{4}$"))]
#[derive(Debug)]
pub struct PinCode(String);

fn main() {
    let dist: Dist = "11.4".parse().unwrap();
    println!("dist = {}", dist.into_inner());

    let age: Age = "77".parse().unwrap();
    let json = serde_json::to_string(&age).unwrap();
    println!("AGE JSON = {json}");

    let username: Username = "greyblake".parse().unwrap();
    let json = serde_json::to_string(&username).unwrap();
    println!("USERNAME JSON = {json}");

    let dist: Dist = serde_json::from_str("12.339999999999").unwrap();
    println!("Dist = {dist}");

    let name = "Bang".to_string();

    let username = unsafe { Username::new_unchecked(name) };
    println!("{username:#?}");

    let pin_result = PinCode::new("1223  ");
    println!("\npin_result = {pin_result:?}\n");
}
