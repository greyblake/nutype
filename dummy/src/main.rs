use nutype::nutype;

#[nutype(validate(max = 12.34))]
#[derive(FromStr, Display, Clone, Copy, Serialize)]
pub struct Dist(f64);

#[nutype(validate(min = 18, max = 99))]
#[derive(FromStr, Display, Clone, Copy, Serialize)]
pub struct Age(u8);

#[nutype]
#[derive(FromStr, Display, Clone, Serialize)]
pub struct Username(String);

fn main() {
    let dist: Dist = "11.4".parse().unwrap();
    println!("dist = {}", dist.into_inner());

    let age: Age = "77".parse().unwrap();
    let json = serde_json::to_string(&age).unwrap();
    println!("AGE JSON = {json}");

    let username: Username = "greyblake".parse().unwrap();
    let json = serde_json::to_string(&username).unwrap();
    println!("USERNAME JSON = {json}");
}
