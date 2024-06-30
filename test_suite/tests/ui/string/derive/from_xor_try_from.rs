use nutype::nutype;

#[nutype(derive(From, TryFrom))]
struct Text(String);

fn main() {}
