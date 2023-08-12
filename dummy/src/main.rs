use nutype::nutype;

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, char_len_max = 255, char_len_min = 6),
    derive(
        TryFrom, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromStr, AsRef, Hash
    )
)]
pub struct Email(String);

#[nutype(
    validate(min = 100, max = 1000),
    derive(Deref, FromStr)
)]
pub struct Number(i16);

fn main() {
    let magic = Number::new(42);
    assert_eq!(*magic, 42);
}
