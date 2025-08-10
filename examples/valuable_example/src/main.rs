use nutype::nutype;

// IMPORTANT: `valuable` must be declared in dependencies with feature `derive`.
use valuable::Valuable;

#[nutype(derive(Valuable))]
pub struct Age(u32);

#[nutype(derive(Valuable))]
pub struct Temperature(f32);

#[nutype(derive(Valuable))]
pub struct Name(String);

#[derive(Valuable)]
pub struct Sleuth {
    name: String,
    solved_cases: u32,
}

fn main() {
    // Integer
    assert_eq!(format!("{:?}", Age::new(25).as_value()), r#"Age(25)"#);

    // Float
    assert_eq!(
        format!("{:?}", Temperature::new(32.3).as_value()),
        r#"Temperature(32.3)"#
    );

    // String
    assert_eq!(
        format!("{:?}", Name::new("Sherlock".to_owned()).as_value()),
        r#"Name("Sherlock")"#
    );
}
