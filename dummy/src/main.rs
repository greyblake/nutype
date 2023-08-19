use nutype::nutype;

#[nutype(
    validate(predicate = another_predicate)
)]
pub struct Another(Vec<i32>);

fn another_predicate(x: &Vec<i32>) -> bool {
    x.len() > 0
}

fn main() {}
