use nutype::nutype;

const MIN_NAME_LEN: usize = 3;
const MAX_NAME_LEN: usize = 4;

#[nutype(
    validate(
        len_char_min = MIN_NAME_LEN,
        len_char_max = MAX_NAME_LEN
    ),
    derive(Debug, AsRef, PartialEq, ),
)]
pub struct Name(String);

fn main() {
    assert_eq!(Name::new("Bo"), Err(NameError::LenCharMinViolated));
    assert_eq!(Name::new("Julia"), Err(NameError::LenCharMaxViolated));
    assert_eq!(Name::new("Jojo").unwrap().as_ref(), "Jojo");
}
