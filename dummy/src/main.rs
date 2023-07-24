use nutype::nutype;

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty),
    derive(
        TryFrom, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
    )
)]
pub struct Email(String);

#[nutype(derive(Deref))]
pub struct Number(i16);

fn main() {
    let magic = Number::new(42);
    assert_eq!(*magic, 42);
}
