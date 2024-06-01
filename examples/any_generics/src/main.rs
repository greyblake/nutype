use nutype::nutype;
use std::borrow::Cow;

#[nutype(
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug),
)]
struct NotEmpty<T>(Vec<T>);

#[nutype(derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Into,
    From,
    Deref,
    Borrow,
    // TODO
    // AsRef,
    // FromStr,
    // TryFrom,
    // Default,
    // Serialize,
    // Deserialize,
    // Arbitrary,
))]
struct Clarabelle<'b>(Cow<'b, str>);

fn main() {
    {
        let v = NotEmpty::new(vec![1, 2, 3]).unwrap();
        assert_eq!(v.into_inner(), vec![1, 2, 3]);
    }
    {
        let err = NotEmpty::<i32>::new(vec![]).unwrap_err();
        assert_eq!(err, NotEmptyError::PredicateViolated);
    }

    {
        let muu = Clarabelle::new(Cow::Borrowed("Muu"));
        assert_eq!(muu.to_string(), "Muu");
    }
}
