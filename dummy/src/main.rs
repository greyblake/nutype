use nutype::nutype;
use std::cmp::Ord;

#[nutype(
    sanitize(with = |mut v| { v.sort(); v }),
    derive(Debug)
)]
struct SortedVec<T: Ord>(Vec<T>);

fn main() {
    let v = SortedVec::new(vec![10, 3, 5, 2, 4]);
    assert_eq!(v.into_inner(), vec![2, 3, 4, 5, 10]);
}
