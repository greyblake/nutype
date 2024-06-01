use nutype::nutype;
use std::borrow::Cow;

#[nutype(derive(Debug, Display))]
struct Clarabelle<'a>(Cow<'a, str>);

fn main() {
    let clarabelle = Clarabelle::new(Cow::Borrowed("Clarabelle"));
    assert_eq!(clarabelle.to_string(), "Clarabelle");
}
