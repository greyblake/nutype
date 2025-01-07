use nutype::nutype;

#[nutype(
    derive(Debug),
    validate(predicate = |name| !name.trim().is_empty())
)]
pub struct Name<'a>(&'a str);

fn main() {
    let name_error = Name::try_new("  ").unwrap_err();
    assert_eq!(name_error, NameError::PredicateViolated);
}
