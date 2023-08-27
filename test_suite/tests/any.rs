use nutype::nutype;
use test_suite::test_helpers::traits::*;

// Inner custom type, which is unknown to nutype
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
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
        let same_location = location.clone();

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
            assert_eq!(err.to_string(), "Failed to parse Location: Invalid integer");
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
                    "Failed to parse Position: Point must be two comma separated integers"
                );
            }

            {
                let err = "5,5".parse::<Position>().unwrap_err();
                assert_eq!(err.to_string(), "Failed to parse Position: invalid");
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
                    assert_eq!(err.to_string(), "invalid");
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
    }
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
