//! Tests for where clause support in generic newtypes (Issue #160)
//!
//! These tests verify that nutype properly handles `where` clauses,
//! including Higher-Ranked Trait Bounds (HRTB).

mod basic_where_clause {
    use nutype::nutype;

    #[test]
    fn test_simple_where_clause() {
        #[nutype(derive(Debug, Clone))]
        struct Wrapper<T>(T)
        where
            T: Default;

        let w = Wrapper::new(42i32);
        assert_eq!(w.into_inner(), 42);
    }

    #[test]
    fn test_where_clause_with_multiple_predicates() {
        // Test where clause with multiple type parameters
        #[nutype(derive(Debug, Clone))]
        struct Pair<T, U>((T, U))
        where
            T: Clone + Default,
            U: Clone + Default;

        let p: Pair<i32, String> = Pair::new((42, String::from("hello")));
        assert_eq!(p.into_inner(), (42, String::from("hello")));
    }

    #[test]
    fn test_where_clause_with_inline_bounds_combined() {
        // Both inline bounds AND where clause
        #[nutype(derive(Debug, Clone))]
        struct Combined<T: Clone>(T)
        where
            T: Default;

        let c = Combined::new(String::from("test"));
        let cloned = c.clone();
        assert_eq!(cloned.into_inner(), "test");
    }
}

mod hrtb_where_clause {
    use nutype::nutype;

    #[test]
    fn test_hrtb_into_iterator() {
        #[nutype(
            validate(predicate = |c| c.into_iter().next().is_some()),
            derive(Debug)
        )]
        struct NonEmpty<C>(C)
        where
            for<'a> &'a C: IntoIterator;

        let vec = vec![1, 2, 3];
        let non_empty = NonEmpty::try_new(vec).unwrap();
        assert_eq!(non_empty.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_hrtb_validation_failure() {
        #[nutype(
            validate(predicate = |c| c.into_iter().next().is_some()),
            derive(Debug)
        )]
        struct NonEmpty<C>(C)
        where
            for<'a> &'a C: IntoIterator;

        let empty: Vec<i32> = vec![];
        assert!(NonEmpty::try_new(empty).is_err());
    }

    #[test]
    fn test_hrtb_with_clone() {
        #[nutype(derive(Debug, Clone))]
        struct Cloneable<C>(C)
        where
            for<'a> &'a C: IntoIterator,
            C: Clone;

        let c = Cloneable::new(vec![1, 2, 3]);
        let cloned = c.clone();
        assert_eq!(cloned.into_inner(), vec![1, 2, 3]);
    }
}

mod where_clause_with_sanitize {
    use nutype::nutype;

    #[test]
    fn test_sanitize_with_where() {
        #[nutype(
            sanitize(with = |mut v: Vec<T>| { v.sort(); v }),
            derive(Debug)
        )]
        struct Sorted<T>(Vec<T>)
        where
            T: Ord;

        let sorted = Sorted::new(vec![3, 1, 2]);
        assert_eq!(sorted.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_sanitize_and_validate_with_where() {
        #[nutype(
            sanitize(with = |mut v: Vec<T>| { v.sort(); v }),
            validate(predicate = |v| !v.is_empty()),
            derive(Debug)
        )]
        struct SortedNonEmpty<T>(Vec<T>)
        where
            T: Ord;

        let sorted = SortedNonEmpty::try_new(vec![3, 1, 2]).unwrap();
        assert_eq!(sorted.into_inner(), vec![1, 2, 3]);

        // Empty should fail validation
        let empty: Vec<i32> = vec![];
        assert!(SortedNonEmpty::try_new(empty).is_err());
    }
}

mod where_clause_with_traits {
    use nutype::nutype;
    use std::fmt::Display;

    #[test]
    fn test_display_with_where() {
        #[nutype(derive(Debug, Display))]
        struct ShowIt<T>(T)
        where
            T: Display;

        let s = ShowIt::new("hello");
        assert_eq!(format!("{}", s), "hello");
    }

    #[test]
    fn test_as_ref_with_where() {
        #[nutype(derive(Debug, AsRef))]
        struct RefIt<T>(T)
        where
            T: Clone;

        let r = RefIt::new(String::from("test"));
        assert_eq!(r.as_ref(), "test");
    }

    #[test]
    fn test_deref_with_where() {
        use std::ops::Deref;

        #[nutype(derive(Debug, Deref))]
        struct DerefIt<T>(T)
        where
            T: Clone;

        let d = DerefIt::new(String::from("test"));
        assert_eq!(d.deref(), "test");
    }

    #[test]
    fn test_borrow_with_where() {
        use std::borrow::Borrow;

        #[nutype(derive(Debug, Borrow))]
        struct BorrowIt<T>(Vec<T>)
        where
            T: Clone;

        let b = BorrowIt::new(vec![1, 2, 3]);
        let borrowed: &Vec<i32> = b.borrow();
        assert_eq!(borrowed, &vec![1, 2, 3]);
    }

    #[test]
    fn test_from_with_where() {
        #[nutype(derive(Debug, From))]
        struct FromIt<T>(Vec<T>)
        where
            T: Clone;

        let f: FromIt<i32> = vec![1, 2, 3].into();
        assert_eq!(f.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_try_from_with_where_and_validation() {
        #[nutype(
            validate(predicate = |v| !v.is_empty()),
            derive(Debug, TryFrom)
        )]
        struct NonEmptyVec<T>(Vec<T>)
        where
            T: Clone;

        use std::convert::TryFrom;
        let f = NonEmptyVec::try_from(vec![1, 2, 3]).unwrap();
        assert_eq!(f.into_inner(), vec![1, 2, 3]);

        let empty: Vec<i32> = vec![];
        assert!(NonEmptyVec::try_from(empty).is_err());
    }

    #[test]
    fn test_into_with_where() {
        #[nutype(derive(Debug, Into))]
        struct IntoIt<T>(Vec<T>)
        where
            T: Clone;

        let i = IntoIt::new(vec![1, 2, 3]);
        let v: Vec<i32> = i.into();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_default_with_where() {
        #[nutype(derive(Debug, Default), default = vec![])]
        struct DefaultVec<T>(Vec<T>)
        where
            T: Default;

        let d: DefaultVec<i32> = DefaultVec::default();
        assert_eq!(d.into_inner(), Vec::<i32>::new());
    }

    #[test]
    fn test_default_with_where_and_validation() {
        #[nutype(
            validate(predicate = |v| v.len() <= 10),
            derive(Debug, Default),
            default = vec![]
        )]
        struct BoundedVec<T>(Vec<T>)
        where
            T: Default;

        let d: BoundedVec<i32> = BoundedVec::default();
        assert_eq!(d.into_inner(), Vec::<i32>::new());
    }
}

#[cfg(feature = "serde")]
mod where_clause_with_serde {
    use nutype::nutype;
    use serde::Serialize;

    #[test]
    fn test_serialize_with_where() {
        #[nutype(derive(Debug, Serialize))]
        struct SerIt<T>(Vec<T>)
        where
            T: Serialize + Clone;

        let s = SerIt::new(vec![1, 2, 3]);
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "[1,2,3]");
    }

    #[test]
    fn test_deserialize_with_simple_where() {
        // Note: Using simple Clone bound - the Deserialize bound is added by nutype
        #[nutype(derive(Debug, Deserialize))]
        struct DeIt<T>(Vec<T>)
        where
            T: Clone;

        let d: DeIt<i32> = serde_json::from_str("[1,2,3]").unwrap();
        assert_eq!(d.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_serde_roundtrip_with_where() {
        // Note: Using simple Clone bound - Serialize/Deserialize bounds are added by nutype
        #[nutype(derive(Debug, Clone, Serialize, Deserialize))]
        struct RoundTrip<T>(Vec<T>)
        where
            T: Clone;

        let original = RoundTrip::new(vec![1, 2, 3]);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RoundTrip<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_serde_with_validation_and_where() {
        #[nutype(
            validate(predicate = |v| !v.is_empty()),
            derive(Debug, Serialize, Deserialize)
        )]
        struct NonEmptySerde<T>(Vec<T>)
        where
            T: Clone;

        let original = NonEmptySerde::try_new(vec![1, 2, 3]).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: NonEmptySerde<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn test_serialize_with_non_generic_where() {
        // Test where clause on non-generic type (using static bound)
        #[nutype(derive(Debug, Serialize))]
        struct StaticStr(String)
        where
            String: Clone;

        let s = StaticStr::new(String::from("hello"));
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"hello\"");
    }
}

mod into_iterator_with_where {
    use nutype::nutype;

    #[test]
    fn test_into_iterator_with_where_clause() {
        #[nutype(derive(Debug, IntoIterator))]
        struct IterableVec<T>(Vec<T>)
        where
            T: Clone;

        let v = IterableVec::new(vec![1, 2, 3]);
        let collected: Vec<i32> = v.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_iterator_ref_with_where_clause() {
        #[nutype(derive(Debug, IntoIterator))]
        struct IterableVec<T>(Vec<T>)
        where
            T: Clone;

        let v = IterableVec::new(vec![1, 2, 3]);
        let collected: Vec<&i32> = (&v).into_iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }
}
