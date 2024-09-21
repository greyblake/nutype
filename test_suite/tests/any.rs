use nutype::nutype;
use std::borrow::Cow;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use test_suite::test_helpers::traits::*;

// Inner custom type, which is unknown to nutype
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[cfg(test)]
    fn magnitude(&self) -> f64 {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        f64::sqrt(x * x + y * y)
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl std::str::FromStr for Point {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .split(',')
            .map(|part| part.parse::<i32>().map_err(|_| "Invalid integer"))
            .collect::<Result<Vec<_>, _>>()?;

        if items.len() != 2 {
            return Err("Point must be two comma separated integers");
        }
        Ok(Point::new(items[0], items[1]))
    }
}

#[nutype(derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, AsRef, Into, From, Deref, Borrow,
    FromStr, Hash
))]
pub struct Location(Point);

mod traits {
    use super::*;

    #[test]
    fn test_debug() {
        let location = Location::new(Point::new(1, 5));

        assert_eq!(format!("{location:?}"), "Location(Point { x: 1, y: 5 })");
    }

    #[test]
    fn test_clone() {
        let location = Location::new(Point::new(5, 8));
        let same_location = location;

        assert_eq!(location.into_inner(), same_location.into_inner(),);
    }

    #[test]
    fn test_copy() {
        let location = Location::new(Point::new(33, 44));
        let copied_location = location;

        assert_eq!(location.into_inner(), copied_location.into_inner(),);
    }

    #[test]
    fn test_partial_eq() {
        let loc1 = Location::new(Point::new(13, 14));
        let loc2 = Location::new(Point::new(13, 14));

        assert!(loc1.eq(&loc2));
    }

    #[test]
    fn test_eq() {
        should_implement_eq::<Location>();
    }

    #[test]
    fn test_partial_ord() {
        use std::cmp::Ordering;

        let loc1 = Location::new(Point::new(1, 1));
        let loc2 = Location::new(Point::new(1, 2));

        assert_eq!(loc1.partial_cmp(&loc1), Some(Ordering::Equal),);
        assert_eq!(loc1.partial_cmp(&loc2), Some(Ordering::Less),);
        assert_eq!(loc2.partial_cmp(&loc1), Some(Ordering::Greater),);
    }

    #[test]
    fn test_ord() {
        use std::cmp::Ordering;

        let loc1 = Location::new(Point::new(1, 1));
        let loc2 = Location::new(Point::new(1, 2));

        assert_eq!(loc1.cmp(&loc2), Ordering::Less,);
    }

    #[test]
    fn test_display() {
        let location = Location::new(Point::new(4, 7));
        assert_eq!(location.to_string(), "4,7");
    }

    #[test]
    fn test_hash() {
        use std::collections::HashMap;
        let mut hashmap: HashMap<Location, i32> = HashMap::new();

        let loc = Location::new(Point::new(3, 4));

        hashmap.insert(loc, 7);
        assert_eq!(hashmap.get(&loc), Some(&7));
    }

    #[test]
    fn test_as_ref() {
        let location = Location::new(Point::new(8, 19));
        assert_eq!(location.as_ref(), &Point::new(8, 19));
    }

    #[test]
    fn test_into() {
        let location = Location::new(Point::new(8, 19));
        let point: Point = location.into();
        assert_eq!(point, Point::new(8, 19));
    }

    #[test]
    fn test_from() {
        let location = Location::from(Point::new(8, 19));
        assert_eq!(location.into_inner(), Point::new(8, 19));
    }

    #[test]
    fn test_deref() {
        // Location does not implement `.magnitude()`, but Point does.
        // In this test we trigger Deref coercion mechanism by calling
        // `.magnitude()` on location.
        let location = Location::from(Point::new(3, 4));
        assert_eq!(location.magnitude(), 5.0);
    }

    #[test]
    fn test_borrow() {
        use std::borrow::Borrow;

        let location = Location::from(Point::new(3, 4));
        let point: &Point = location.borrow();
        assert_eq!(point, &Point::new(3, 4));
    }

    mod from_str {
        use super::*;

        #[test]
        fn test_without_validation() {
            let loc: Location = "3,5".parse().unwrap();
            assert_eq!(loc, Location::new(Point::new(3, 5)));

            let err = "3,lol".parse::<Location>().unwrap_err();
            assert_eq!(
                err.to_string(),
                "Failed to parse Location: \"Invalid integer\""
            );
        }

        #[test]
        fn test_with_validation() {
            #[nutype(
                derive(Debug, FromStr),
                validate(predicate = |p: &Point| p.x > p.y)
            )]
            pub struct Position(Point);

            {
                let pos = "6,5".parse::<Position>().unwrap();
                assert_eq!(pos.into_inner(), Point::new(6, 5));
            }

            {
                let err = "6,5,4".parse::<Position>().unwrap_err();
                assert_eq!(
                    err.to_string(),
                    "Failed to parse Position: \"Point must be two comma separated integers\""
                );
            }

            {
                let err = "5,5".parse::<Position>().unwrap_err();
                assert_eq!(
                    err.to_string(),
                    "Failed to parse Position: Position failed the predicate test."
                );
            }
        }
    }

    mod try_from {
        use super::*;

        #[test]
        fn test_without_validation() {
            // Note: here we're deriving TryFrom without From, because if T implements From, then
            // TryFrom is implemented automatically (blanket implementation)
            #[nutype(derive(Debug, TryFrom))]
            pub struct Destination(Point);

            let dest = Destination::try_from(Point::new(3, 2)).unwrap();
            assert_eq!(dest.into_inner(), Point::new(3, 2));
        }

        #[test]
        fn test_with_validation() {
            #[nutype(
                derive(Debug, TryFrom),
                validate(predicate = |p: &Point| p.x > p.y)
            )]
            pub struct Position(Point);

            {
                let err = Position::try_from(Point::new(2, 2)).unwrap_err();
                assert_eq!(err, PositionError::PredicateViolated);
            }

            {
                let pos = Position::try_from(Point::new(3, 2)).unwrap();
                assert_eq!(pos.into_inner(), Point::new(3, 2));
            }
        }
    }

    #[test]
    fn test_default() {
        #[nutype(
            derive(Debug, Default),
            default = Point { x: 6, y: 9 }
        )]
        pub struct Lugar(Point);

        assert_eq!(Lugar::default().into_inner(), Point::new(6, 9),);
    }

    #[cfg(feature = "serde")]
    mod serialization {
        use super::*;

        #[nutype(derive(Debug, Serialize, Deserialize, PartialEq))]
        struct Place(Point);

        mod json_format {
            use super::*;

            #[test]
            fn test_trait_serialize() {
                let place = Place::new(Point::new(22, 99));

                let place_json = serde_json::to_string(&place).unwrap();
                assert_eq!(place_json, "{\"x\":22,\"y\":99}");
            }

            #[test]
            fn test_trait_deserialize_without_validation() {
                let place: Place = serde_json::from_str("{\"x\":22,\"y\":99}").unwrap();
                assert_eq!(place.into_inner(), Point::new(22, 99));
            }

            #[test]
            fn test_trait_deserialize_with_validation() {
                #[nutype(
                    derive(Deserialize, Debug),
                    validate(predicate = |p: &Point| p.y == p.x ),
                )]
                pub struct LinePoint(Point);

                {
                    let err = serde_json::from_str::<LinePoint>("{\"x\":7,\"y\":9}").unwrap_err();
                    assert_eq!(
                        err.to_string(),
                        "LinePoint failed the predicate test. Expected valid LinePoint"
                    );
                }

                {
                    let lp = serde_json::from_str::<LinePoint>("{\"x\":7,\"y\":7}").unwrap();
                    assert_eq!(lp.into_inner(), Point::new(7, 7));
                }
            }
        }

        mod ron_format {
            use super::*;

            #[test]
            fn test_ron_roundtrip() {
                let place = Place::new(Point::new(44, 55));

                let serialized = ron::to_string(&place).unwrap();
                let deserialized: Place = ron::from_str(&serialized).unwrap();

                assert_eq!(deserialized, place);
            }
        }

        mod message_pack_format {
            use super::*;

            #[test]
            fn test_rmp_roundtrip() {
                let place = Place::new(Point::new(44, 55));

                let bytes = rmp_serde::to_vec(&place).unwrap();
                let deserialized: Place = rmp_serde::from_slice(&bytes).unwrap();

                assert_eq!(deserialized, place);
            }
        }

        mod with_generics {
            use super::*;

            #[test]
            fn test_generic_with_serde() {
                #[nutype(
                    derive(Debug, Serialize, Deserialize),
                    validate(predicate = |v| !v.is_empty())
                )]
                struct NonEmptyVec<T>(Vec<T>);

                {
                    let result = NonEmptyVec::<i32>::try_new(vec![]);
                    assert!(result.is_err());
                }

                {
                    let nev = NonEmptyVec::try_new(vec![5, 2, 3]).unwrap();
                    assert_eq!(nev.into_inner(), vec![5, 2, 3],);
                }
            }

            #[test]
            fn serialize_and_deserialize_cow() {
                #[nutype(
                    validate(predicate = |s| s.len() >= 3),
                    derive(Debug, Serialize, Deserialize, PartialEq)
                )]
                struct Clarabelle<'a>(Cow<'a, str>);

                let muu = Clarabelle::try_new(Cow::Borrowed("Muu")).unwrap();
                let json = serde_json::to_string(&muu).unwrap();
                assert_eq!(json, "\"Muu\"");
                let same_muu: Clarabelle = serde_json::from_str(&json).unwrap();
                assert_eq!(muu, same_muu);
            }
        }
    }
}

#[test]
fn test_sanitize_and_validate_with_untyped_closure() {
    #[nutype(
        derive(Debug),
        sanitize(with = |p| {
            Point {
                x: p.x.clamp(0, 100),
                y: p.y.clamp(0, 100),
            }
        }),
        validate(predicate = |p| p.x > p.y),
    )]
    pub struct Pos(Point);

    let pos = Pos::try_new(Point::new(123, 91)).unwrap();
    assert_eq!(pos.into_inner(), Point::new(100, 91))
}

#[test]
fn test_sanitize_with_untyped_mut_closure() {
    #[nutype(
        derive(Debug),
        sanitize(with = |mut p| {
            p.x = p.x.clamp(0, 100);
            p.y = p.y.clamp(0, 100);
            p
        }),
    )]
    pub struct Pos(Point);

    let pos = Pos::new(Point::new(123, 91));
    assert_eq!(pos.into_inner(), Point::new(100, 91))
}

#[cfg(test)]
#[cfg(feature = "new_unchecked")]
mod new_unchecked {
    use super::*;

    #[nutype(
        derive(Debug),
        validate(predicate = |p: &Point| p.y == p.x ),
        new_unchecked,
    )]
    pub struct LinePoint(Point);

    #[test]
    fn test_new_unchecked() {
        let line_point = unsafe { LinePoint::new_unchecked(Point::new(3, 4)) };
        assert_eq!(line_point.into_inner(), Point::new(3, 4));
    }
}

#[cfg(test)]
mod with_generics {
    use super::*;

    #[test]
    fn test_generic_with_validate() {
        #[nutype(
            validate(predicate = |v| !v.is_empty()),
            derive(Debug)
        )]
        struct NonEmptyVec<T>(Vec<T>);

        {
            let vec = NonEmptyVec::try_new(vec![1, 2, 3]).unwrap();
            assert_eq!(vec.into_inner(), vec![1, 2, 3]);
        }

        {
            let vec = NonEmptyVec::try_new(vec![5]).unwrap();
            assert_eq!(vec.into_inner(), vec![5]);
        }

        {
            let vec: Vec<u8> = vec![];
            let err = NonEmptyVec::try_new(vec).unwrap_err();
            assert_eq!(err, NonEmptyVecError::PredicateViolated);
        }
    }

    #[test]
    fn test_generic_with_sanitize() {
        #[nutype(
            sanitize(with = |mut v| { v.truncate(2); v }),
            derive(Debug)
        )]
        struct UpToTwo<T>(Vec<T>);

        {
            let vec = UpToTwo::new(vec![1, 2, 3]);
            assert_eq!(vec.into_inner(), vec![1, 2]);
        }

        {
            let vec = UpToTwo::new(vec![5]);
            assert_eq!(vec.into_inner(), vec![5]);
        }
    }

    #[test]
    fn test_generic_with_sanitize_and_validate() {
        #[nutype(
            sanitize(with = |mut v| { v.truncate(2); v }),
            validate(predicate = |v| !v.is_empty()),
            derive(Debug)
        )]
        struct OneOrTwo<T>(Vec<T>);

        {
            let vec = OneOrTwo::try_new(vec![1, 2, 3]).unwrap();
            assert_eq!(vec.into_inner(), vec![1, 2]);
        }

        {
            let vec = OneOrTwo::try_new(vec![5]).unwrap();
            assert_eq!(vec.into_inner(), vec![5]);
        }

        {
            let vec: Vec<u8> = vec![];
            let err = OneOrTwo::try_new(vec).unwrap_err();
            assert_eq!(err, OneOrTwoError::PredicateViolated);
        }
    }

    #[test]
    fn test_generic_with_boundaries_and_sanitize() {
        #[nutype(
            sanitize(with = |mut v| { v.sort(); v }),
            derive(Debug)
        )]
        struct SortedVec<T: Ord>(Vec<T>);

        let sorted = SortedVec::new(vec![3, 1, 2]);
        assert_eq!(sorted.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_generic_with_boundaries_and_many_derives() {
        // The point of this test is to ensure that the generate code can be compiled at least
        // with respect to the specified trait boundaries

        #[nutype(derive(Debug, Clone, PartialEq, Eq))]
        struct Wrapper1<A: Hash + Eq + Clone, B: Ord>(HashMap<A, B>);
    }

    #[test]
    fn test_generic_boundaries_debug() {
        #[nutype(derive(Debug))]
        struct WrapperDebug<T>(T);

        let w = WrapperDebug::new(13);
        assert_eq!(format!("{w:?}"), "WrapperDebug(13)");
    }

    #[test]
    fn test_generic_boundaries_display() {
        #[nutype(derive(Debug, Display))]
        struct WrapperDisplay<T>(T);

        let number = WrapperDisplay::new(5);
        assert_eq!(number.to_string(), "5");

        let b = WrapperDisplay::new(true);
        assert_eq!(b.to_string(), "true");
    }

    #[test]
    fn test_generic_boundaries_clone() {
        #[nutype(derive(Clone))]
        struct WrapperClone<T>(T);

        let val = WrapperClone::new(17);
        let cloned = val.clone();
        assert_eq!(val.into_inner(), cloned.into_inner());
    }

    #[test]
    fn test_generic_boundaries_copy() {
        #[nutype(derive(Clone, Copy))]
        struct WrapperCopy<T>(T);

        let val = WrapperCopy::new(17);
        let copied = val;
        assert_eq!(val.into_inner(), copied.into_inner());
    }

    #[test]
    fn test_generic_boundaries_partial_eq_and_eq() {
        #[nutype(derive(PartialEq, Eq))]
        struct WrapperPartialEq<T: PartialEq + Eq>(T);

        let v19 = WrapperPartialEq::new(19);
        let v19x = WrapperPartialEq::new(19);
        let v3 = WrapperPartialEq::new(20);
        assert!(v19.eq(&v19x));
        assert!(v19.ne(&v3));
    }

    #[test]
    fn test_generic_boundaries_partial_ord() {
        #[nutype(derive(PartialEq, PartialOrd))]
        struct WrapperPartialOrd<T>(T);

        {
            let v1 = WrapperPartialOrd::new(1);
            let v2 = WrapperPartialOrd::new(2);
            let v2x = WrapperPartialOrd::new(2);

            assert_eq!(v1.partial_cmp(&v2).unwrap(), Ordering::Less);
            assert_eq!(v2.partial_cmp(&v2x).unwrap(), Ordering::Equal);
        }

        {
            let nan1 = WrapperPartialOrd::new(std::f64::NAN);
            let nan2 = WrapperPartialOrd::new(std::f64::NAN);
            assert_eq!(nan1.partial_cmp(&nan2), None);
        }
    }

    #[test]
    fn test_generic_boundaries_ord() {
        #[nutype(derive(PartialEq, Eq, PartialOrd, Ord))]
        struct WrapperOrd<T>(T);

        let v1 = WrapperOrd::new(1);
        let v2 = WrapperOrd::new(2);
        let v2x = WrapperOrd::new(2);

        assert_eq!(v2.cmp(&v1), Ordering::Greater);
        assert_eq!(v2.cmp(&v2x), Ordering::Equal);
    }

    #[test]
    fn test_generic_boundaries_hash() {
        #[nutype(derive(PartialEq, Eq, Hash))]
        struct WrapperHash<T: PartialEq + Eq + Hash>(T);

        #[derive(Hash, PartialEq, Eq)]
        struct Number(i32);

        let mut set = HashSet::new();
        set.insert(WrapperHash::new(Number(1)));
        set.insert(WrapperHash::new(Number(1)));
        set.insert(WrapperHash::new(Number(2)));

        // 1 is inserted twice, so the set should have only two elements
        assert_eq!(set.len(), 2);
    }

    #[cfg(feature = "serde")]
    mod serialization_with_generics {
        use super::*;

        #[test]
        fn test_serialize() {
            #[nutype(derive(Debug, Serialize))]
            struct Wrapper<T>(T);

            let w = Wrapper::new(13);
            let json = serde_json::to_string(&w).unwrap();
            assert_eq!(json, "13");
        }

        #[test]
        fn test_deserialize() {
            #[nutype(derive(Debug, Deserialize, PartialEq, Eq))]
            struct Wrapper<T>(T);

            let json = "14";
            let w = serde_json::from_str::<Wrapper<u8>>(json).unwrap();
            assert_eq!(w, Wrapper::new(14));
        }

        #[test]
        fn test_serialize_and_deserialize_type_with_sanitization_and_validations() {
            #[nutype(
                sanitize(with = |mut v| { v.sort(); v }),
                validate(predicate = |vec| !vec.is_empty()),
                derive(Debug, Deserialize, Serialize),
            )]
            struct SortedNotEmptyVec<T: Ord>(Vec<T>);

            let input_json = "[3, 1, 5, 2]";
            let snev = serde_json::from_str::<SortedNotEmptyVec<i32>>(input_json).unwrap();
            let output_json = serde_json::to_string(&snev).unwrap();
            assert_eq!(output_json, "[1,2,3,5]");
        }
    }

    #[test]
    fn test_generic_try_from_without_validation() {
        // Note, that we get TryFrom thanks to the blanket implementation in core:
        //
        //    impl<T, U> TryFrom<U> for T
        //    where
        //       U: Into<T>
        //
        #[nutype(derive(Debug, From))]
        struct Doener<T>(T);

        let durum = Doener::from("Durum");
        assert_eq!(durum.into_inner(), "Durum");
    }

    #[test]
    fn test_generic_try_from_with_validation() {
        #[nutype(
            derive(Debug, TryFrom),
            validate(predicate = |v| !v.is_empty())
        )]
        struct NotEmpty<T>(Vec<T>);
        {
            let err = NotEmpty::<i32>::try_from(vec![]).unwrap_err();
            assert_eq!(err, NotEmptyError::PredicateViolated);
        }
        {
            let v = NotEmpty::try_from(vec![1, 2, 3]).unwrap();
            assert_eq!(v.into_inner(), vec![1, 2, 3]);
        }
    }

    #[test]
    fn test_generic_from_with_bounds_and_sanitization() {
        #[nutype(
            sanitize(with = |mut v| { v.sort(); v }),
            derive(Debug, From),
        )]
        struct Sorted<T: Ord>(Vec<T>);

        let sorted: Sorted<i32> = Sorted::from(vec![3, 1, 2]);
        assert_eq!(sorted.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_generic_from_str_without_validation() {
        // Note: the code generate for FromStr relies on "associated type bounds" feature, which is
        // stabilized only in 1.79.
        #[nutype(derive(Debug, FromStr))]
        struct Parseable<T>(T);

        {
            let xiii = "13".parse::<Parseable<i32>>().unwrap();
            assert_eq!(xiii.into_inner(), 13);
        }

        {
            let vii = "vii".parse::<Parseable<String>>().unwrap();
            assert_eq!(vii.into_inner(), "vii");
        }

        {
            let err = "iv".parse::<Parseable<i32>>().unwrap_err();
            assert_eq!(
                err.to_string(),
                "Failed to parse Parseable: ParseIntError { kind: InvalidDigit }"
            );
        }

        {
            let four = "4".parse::<Parseable<Parseable<i32>>>().unwrap();
            assert_eq!(four.into_inner().into_inner(), 4);
        }
    }

    #[test]
    fn test_generic_from_str_with_validation() {
        #[nutype(
            validate(predicate = |n| n.is_even()),
            derive(Debug, FromStr),
        )]
        struct Even<T: ::num::Integer>(T);

        {
            let err = "13".parse::<Even<i32>>().unwrap_err();
            assert_eq!(
                err.to_string(),
                "Failed to parse Even: Even failed the predicate test."
            );
        }

        {
            let twelve = "12".parse::<Even<i32>>().unwrap();
            assert_eq!(twelve.into_inner(), 12);
        }
    }

    mod generics_and_arbitrary {
        use super::*;
        use arbitrary::Arbitrary;

        #[nutype(derive(Debug, Arbitrary))]
        struct Arbaro<T>(Vec<T>);

        fn gen(bytes: &[u8]) -> Vec<bool> {
            let mut u = arbitrary::Unstructured::new(bytes);
            let arbraro = Arbaro::<bool>::arbitrary(&mut u).unwrap();
            arbraro.into_inner()
        }

        #[test]
        fn test_generic_boundaries_arbitrary() {
            assert_eq!(gen(&[]), Vec::<bool>::new());
            assert_eq!(gen(&[1]), vec![false]);
            assert_eq!(gen(&[1, 3, 5]), vec![true, false]);
        }
    }

    #[test]
    fn test_generic_with_boundaries_and_sanitize_and_validate() {
        #[nutype(
            validate(predicate = |v| !v.is_empty()),
            sanitize(with = |mut v| { v.sort(); v }),
            derive(Debug)
        )]
        struct NonEmptySortedVec<T: Ord>(Vec<T>);

        {
            let err = NonEmptySortedVec::<i32>::try_new(vec![]).unwrap_err();
            assert_eq!(err, NonEmptySortedVecError::PredicateViolated);
        }
        {
            let vec = NonEmptySortedVec::try_new(vec![3, 1, 2]).unwrap();
            assert_eq!(vec.into_inner(), vec![1, 2, 3]);
        }
    }

    #[test]
    fn test_generic_with_lifetime_cow() {
        #[nutype(
            validate(predicate = |s| s.len() >= 3),
            derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, Deref, Borrow, TryFrom, AsRef)
        )]
        struct Clarabelle<'a>(Cow<'a, str>);

        {
            let clarabelle = Clarabelle::try_new(Cow::Borrowed("Clarabelle")).unwrap();
            assert_eq!(clarabelle.to_string(), "Clarabelle");

            let muu = Clarabelle::try_new(Cow::Owned("Muu".to_string())).unwrap();
            assert_eq!(muu.to_string(), "Muu");
        }
    }

    #[test]
    fn test_derive_as_ref_with_generic() {
        #[nutype(derive(AsRef))]
        struct SquareMap<K, V>(HashMap<K, V>);

        let mut inner_map = HashMap::new();
        inner_map.insert(4, 16);
        inner_map.insert(5, 25);
        let squares = SquareMap::new(inner_map.clone());
        assert_eq!(squares.as_ref(), &inner_map);
    }

    #[test]
    fn test_derive_as_ref_with_generic_and_validation() {
        #[nutype(
            validate(predicate = |map| map.len() > 1),
            derive(AsRef)
        )]
        struct NonEmptyMap<K, V>(HashMap<K, V>);

        let mut inner_map = HashMap::new();
        inner_map.insert(4, 16);
        inner_map.insert(5, 25);
        let squares = NonEmptyMap::try_new(inner_map.clone()).unwrap();
        assert_eq!(squares.as_ref(), &inner_map);
    }

    #[test]
    fn test_derive_as_ref_with_generic_boundaries_and_validation_and_sanitization() {
        #[nutype(
            sanitize(with = |mut v| { v.sort(); v }),
            validate(predicate = |vec| !vec.is_empty()),
            derive(Debug, AsRef, PartialEq),
        )]
        struct Friends<T: Ord>(Vec<T>);

        assert_eq!(
            Friends::<&str>::try_new(vec![]),
            Err(FriendsError::PredicateViolated)
        );

        let wise_friends = Friends::try_new(vec!["Seneca", "Zeno", "Aristotle"]).unwrap();
        assert_eq!(wise_friends.as_ref(), &["Aristotle", "Seneca", "Zeno"]);
    }

    #[test]
    fn test_derive_deref_with_generic_boundaries_and_validation_and_sanitization() {
        #[nutype(
            sanitize(with = |mut v| { v.sort(); v }),
            validate(predicate = |vec| !vec.is_empty()),
            derive(Debug, Deref),
        )]
        struct Penguins<T: Ord>(Vec<T>);

        let penguins = Penguins::try_new(vec!["Tux", "Chilly Willy"]).unwrap();
        assert_eq!(penguins.len(), 2);
    }

    #[test]
    fn test_derive_borrow_with_generic_boundaries_and_validation_and_sanitization() {
        use std::borrow::Borrow;

        #[nutype(
            sanitize(with = |mut v| { v.sort(); v }),
            validate(predicate = |vec| !vec.is_empty()),
            derive(Debug, Borrow),
        )]
        struct Heroes<T: Ord>(Vec<T>);

        let heroes = Heroes::try_new(vec!["Spiderman", "Batman"]).unwrap();
        let borrowed_heroes: &Vec<&str> = heroes.borrow();
        assert_eq!(borrowed_heroes, &vec!["Batman", "Spiderman"]);
    }

    #[test]
    fn test_derive_default_with_generics() {
        #[nutype(
            derive(Debug, Default),
            default = vec![T::default()],
        )]
        struct Collection<T: Default>(Vec<T>);

        let ints = Collection::<u32>::default();
        assert_eq!(ints.into_inner(), vec![0]);

        let bools = Collection::<bool>::default();
        assert_eq!(bools.into_inner(), vec![false]);
    }

    #[test]
    fn test_derive_default_with_generics_and_validation() {
        #[nutype(
            derive(Debug, Default),
            default = vec![T::default()],
            validate(predicate = |c| !c.is_empty()),
        )]
        struct Collection<T: Default>(Vec<T>);

        let collection = Collection::<u32>::default();
        assert_eq!(collection.into_inner(), vec![0]);
    }
}

mod custom_error {
    use super::*;
    use thiserror::Error;

    #[nutype(
        validate(with = validate_decent_collection, error = namespaced_error::DecentCollectionError),
        derive(Debug, PartialEq, AsRef),
    )]
    struct DecentCollection<T>(Vec<T>);

    fn validate_decent_collection<T>(
        collection: &[T],
    ) -> Result<(), namespaced_error::DecentCollectionError> {
        use namespaced_error::DecentCollectionError;

        if collection.len() < 3 {
            Err(DecentCollectionError::TooShort)
        } else if collection.len() > 10 {
            Err(DecentCollectionError::TooLong)
        } else {
            Ok(())
        }
    }

    // NOTE: The error is within the module is on purpose to ensure that `error = namespaced_error::DecentCollectionError`
    // works as expected.
    mod namespaced_error {
        use super::*;

        #[derive(Error, Debug, PartialEq)]
        pub enum DecentCollectionError {
            #[error("Collection is too short.")]
            TooShort,

            #[error("Collection is too long.")]
            TooLong,
        }
    }

    #[test]
    fn test_custom_error() {
        use namespaced_error::DecentCollectionError;

        assert_eq!(
            DecentCollection::try_new(vec![1, 2]),
            Err(DecentCollectionError::TooShort)
        );

        assert_eq!(
            DecentCollection::try_new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
            Err(DecentCollectionError::TooLong)
        );

        let collection = DecentCollection::try_new(vec![1, 2, 3]).unwrap();
        assert_eq!(collection.as_ref(), &[1, 2, 3]);
    }
}
