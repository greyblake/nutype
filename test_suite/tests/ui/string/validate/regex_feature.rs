use nutype::nutype;

#[nutype(validate(regex = "^[0-9]{3}-[0-9]{3}$"))]
struct PhoneNumber(String);

fn main() {}
