use nutype::nutype;
use std::cmp::Ord;

#[nutype(
    sanitize(with = |mut v| { v.sort(); v }),
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug, Deserialize, Serialize),
)]
struct SortedNotEmptyVec<T: Ord>(Vec<T>);

fn main() {
    {
        // Not empty vec is fine
        let json = "[3, 1, 5, 2]";
        let sv = serde_json::from_str::<SortedNotEmptyVec<i32>>(json).unwrap();
        assert_eq!(sv.into_inner(), vec![1, 2, 3, 5]);
    }
    {
        // Empty vec is not allowed
        let json = "[]";
        let result = serde_json::from_str::<SortedNotEmptyVec<i32>>(json);
        assert!(result.is_err());
    }
}
