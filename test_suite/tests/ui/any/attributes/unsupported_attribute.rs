use nutype::nutype;

pub struct Inner(String);

#[nutype(checked_ops)]
pub struct Name(Inner);

fn main () {}
