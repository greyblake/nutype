use nutype::nutype;
use std::borrow::Cow;

/// A wrapper around a vector that is guaranteed to be sorted.
#[nutype(
    sanitize(with = |mut v| { v.sort(); v }),
    derive(Debug)
)]
struct SortedVec<T: Ord>(Vec<T>);

/// A wrapper around a vector that is guaranteed to be non-empty.
#[nutype(
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug),
)]
struct NotEmpty<T>(Vec<T>);

/// An example with lifetimes
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
    AsRef,
    Serialize,
    Deserialize,
    // TODO
    // FromStr,
    // TryFrom,
    // Default,
    // Arbitrary,
))]
struct Clarabelle<'a>(Cow<'a, str>);

fn main() {
    {
        let v = SortedVec::new(vec![3, 0, 2]);
        assert_eq!(v.into_inner(), vec![0, 2, 3]);
    }
    {
        let v = NotEmpty::new(vec![1, 2, 3]).unwrap();
        assert_eq!(v.into_inner(), vec![1, 2, 3]);

        let err = NotEmpty::<i32>::new(vec![]).unwrap_err();
        assert_eq!(err, NotEmptyError::PredicateViolated);
    }

    {
        let muu = Clarabelle::new(Cow::Borrowed("Muu"));
        assert_eq!(muu.to_string(), "Muu");
        // serialize muu with serde_json
        let json = serde_json::to_string(&muu).unwrap();
        assert_eq!(json, "\"Muu\"");
        // deserialize muu with serde_json
        let same_muu: Clarabelle = serde_json::from_str(&json).unwrap();
        assert_eq!(muu, same_muu);
    }
}
