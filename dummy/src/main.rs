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
    derive(Deref, FromStr),
    validate(greater_or_equal = 10, less_or_equal = 1000)
)]
pub struct Number(i16);

fn main() {
    let magic = Number::new(42).unwrap();
    assert_eq!(*magic, 42);
}
