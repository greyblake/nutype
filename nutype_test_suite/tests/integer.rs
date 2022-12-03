use nutype::nutype;

#[cfg(test)]
mod traits {
    use super::*;
    // use nutype_test_suite::test_helpers::traits::*;

    #[test]
    fn test_trait_into() {
        #[nutype]
        #[derive(Into)]
        pub struct Age(u8);

        let age = Age::new(32);
        let age: u8 = age.into();
        assert_eq!(age, 32);
    }

    #[test]
    fn test_trait_from() {
        #[nutype]
        #[derive(From)]
        pub struct Amount(u32);

        let amount = Amount::from(350);
        assert_eq!(amount.into_inner(), 350);
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype]
        #[derive(AsRef)]
        pub struct Age(u8);

        let age = Age::new(32);
        let age_ref: &u8 = age.as_ref();
        assert_eq!(age_ref, &32);
    }

    #[test]
    fn test_trait_borrow() {
        use std::borrow::Borrow;

        #[nutype]
        #[derive(Borrow)]
        pub struct Age(u8);

        let age = Age::new(32);
        let age_borrowed: &u8 = age.borrow();
        assert_eq!(age_borrowed, &32);
    }

    #[test]
    fn test_trait_try_from() {
        #[nutype(validate(max = 1000))]
        #[derive(Debug, TryFrom)]
        pub struct Amount(i64);

        let amount = Amount::try_from(1000).unwrap();
        assert_eq!(amount.into_inner(), 1000);

        let error = Amount::try_from(1001).unwrap_err();
        assert_eq!(error, AmountError::TooBig);
    }
}
