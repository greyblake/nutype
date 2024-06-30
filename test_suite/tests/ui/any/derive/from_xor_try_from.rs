use nutype::nutype;

#[nutype(derive(From, TryFrom))]
struct Items<T>(T);

fn main() {}
