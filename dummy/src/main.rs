use nutype::nutype;

#[nutype(
    derive_unchecked(::std::fmt::Debug),
    validate(len_char_max = 5, len_char_min = 3)
)]
struct Name(String);

fn main() {
    let err = Name::try_new("Alissa").unwrap_err();
    println!("{err}");
}
