use nutype::nutype;

#[nutype(validate(present))]
#[derive(From)]
struct Name(String);

fn main() {}
