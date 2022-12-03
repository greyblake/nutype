use nutype::nutype;

#[cfg(test)]
mod traits {
    use super::*;
    use nutype_test_suite::test_helpers::traits::*;

    #[test]
    fn test_without_validation() {
        #[nutype]
        #[derive(Debug, From, FromStr, Borrow)]
        pub struct Dist(f32);

        should_implement_debug::<Dist>();
        should_implement_from::<Dist, f32>();
        should_implement_from_str::<Dist>();
        should_implement_borrow::<Dist, f32>();
    }

    #[test]
    fn test_with_validaiton() {
        #[nutype(validate(max = 100.0))]
        #[derive(Debug, TryFrom, FromStr, Borrow)]
        pub struct Dist(f64);

        should_implement_debug::<Dist>();
        should_implement_try_from::<Dist, f64>();
        // TODO: implement FromStr with validation
        // should_implement_from_str::<Name>();
        should_implement_borrow::<Dist, f64>();
    }

    #[test]
    fn test_trait_into() {
        #[nutype]
        #[derive(Into)]
        pub struct Size(f64);

        let size = Size::new(35.7);
        let size: f64 = size.into();
        assert_eq!(size, 35.7);
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype]
        #[derive(AsRef)]
        pub struct Weight(f32);

        let weight = Weight::new(72.650);
        let weight_ref: &f32 = weight.as_ref();
        assert_eq!(weight_ref, &72.650);
    }
}
