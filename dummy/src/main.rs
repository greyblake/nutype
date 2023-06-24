use nutype::nutype;

#[nutype(
    validate(max_len = 6)
    default = "FooBar"
)]
#[derive(Debug, Display, Default)]
pub struct Name(String);

fn main() {
    let name = Name::default();
    println!("name = {name}");
}
