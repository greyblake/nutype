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
}
