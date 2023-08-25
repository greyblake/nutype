use nutype::nutype;
use test_suite::test_helpers::traits::*;

// Inner custom type, which is unknown to nutype
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn magnitude(&self) -> f64 {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        f64::sqrt(x * x + y * y)
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[nutype(derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, AsRef, Into, From, Deref, Borrow
))]
struct Location(Point);

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
        assert_eq!(location.to_string(), "(4, 7)");
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
}
