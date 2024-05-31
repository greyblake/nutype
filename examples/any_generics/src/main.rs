use nutype::nutype;
use std::borrow::Cow;

#[nutype(
    validate(predicate = |vec| !vec.is_empty()),
    derive(Debug),
)]
struct NotEmpty<T>(Vec<T>);

#[nutype(
    derive(Debug),
    validate(predicate = |s| s.len() >= 3),
)]
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
        let c1 = Clarabelle::new(Cow::Borrowed("Muu")).unwrap();
        assert_eq!(c1.into_inner(), Cow::Borrowed("Muu"));
    }
}
