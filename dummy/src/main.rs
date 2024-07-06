use nutype::nutype;

#[nutype(
    sanitize(with = |mut v| { v.sort(); v }),
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug, AsRef, PartialEq, Deref),
)]
struct SortedNotEmptyVec<T: Ord>(Vec<T>);

fn main() {
    // Empty list is not allowed
    assert_eq!(
        SortedNotEmptyVec::<&str>::try_new(vec![]),
        Err(SortedNotEmptyVecError::PredicateViolated)
    );

    let wise_friends = SortedNotEmptyVec::try_new(vec!["Seneca", "Zeno", "Plato"]).unwrap();
    assert_eq!(wise_friends.as_ref(), &["Plato", "Seneca", "Zeno"]);
    assert_eq!(wise_friends.len(), 3);

    let numbers = SortedNotEmptyVec::try_new(vec![4, 2, 7, 1]).unwrap();
    assert_eq!(numbers.as_ref(), &[1, 2, 4, 7]);
    assert_eq!(numbers.len(), 4);
}
