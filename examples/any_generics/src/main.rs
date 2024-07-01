use nutype::nutype;
use std::borrow::Cow;
use std::cmp::Ord;

/// A wrapper around a vector that is guaranteed to be sorted.
#[nutype(
    sanitize(with = |mut v| { v.sort(); v }),
    derive(Debug, Deserialize, Serialize)
)]
struct SortedVec<T: Ord>(Vec<T>);

/// A wrapper around a vector that is guaranteed to be non-empty.
#[nutype(
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug),
)]
struct NotEmpty<T>(Vec<T>);

#[nutype(
    sanitize(with = |mut v| { v.sort(); v }),
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug, Deserialize, Serialize),
)]
struct SortedNotEmptyVec<T: Ord>(Vec<T>);

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
    // SortedVec
    //
    {
        let v = SortedVec::new(vec![3, 0, 2]);
        assert_eq!(v.into_inner(), vec![0, 2, 3]);
    }
    {
        let sv = SortedVec::new(vec![4i32, 2, 8, 5]);
        let json = serde_json::to_string(&sv).unwrap();
        assert_eq!(json, "[2,4,5,8]");
    }
    {
        let json = "[5,3,7]";
        let sv = serde_json::from_str::<SortedVec<i32>>(json).unwrap();
        assert_eq!(sv.into_inner(), vec![3, 5, 7]);
    }

    // NotEmpty
    //
    {
        let v = NotEmpty::try_new(vec![1, 2, 3]).unwrap();
        assert_eq!(v.into_inner(), vec![1, 2, 3]);

        let err = NotEmpty::<i32>::try_new(vec![]).unwrap_err();
        assert_eq!(err, NotEmptyError::PredicateViolated);
    }

    // SortedNotEmptyVec
    //
    {
        // Not empty vec is fine
        let json = "[3, 1, 5, 2]";
        let snev = serde_json::from_str::<SortedNotEmptyVec<i32>>(json).unwrap();
        assert_eq!(snev.into_inner(), vec![1, 2, 3, 5]);
    }
    {
        // Empty vec is not allowed
        let json = "[]";
        let result = serde_json::from_str::<SortedNotEmptyVec<i32>>(json);
        assert!(result.is_err());
    }

    // Clarabelle (Cow)
    //
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
