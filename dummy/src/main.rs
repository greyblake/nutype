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

    let wise_friends =
        SortedNotEmptyVec::try_new(vec!["Seneca", "Zeno", "Socrates", "Epictetus", "Plato"])
            .unwrap();

    assert_eq!(
        wise_friends.as_ref(),
        &["Epictetus", "Plato", "Seneca", "Socrates", "Zeno"]
    );

    assert_eq!(wise_friends.len(), 5);
}
