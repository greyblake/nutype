pub use nutype_derive::nutype;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_example() {
        #[nutype(
             sanitize(trim, lowercase)
             validate(present)
         )]
        #[derive(*)]
        pub struct Email(String);

        let email = Email::new("  OH@my.example\n\n").unwrap();
        assert_eq!(email.into_inner(), "oh@my.example");

        assert_eq!(Email::new("  \n"), Err(EmailError::Missing));
    }

    #[test]
    fn test_amount_example() {
        #[nutype(validate(min = 100, max = 1_000))]
        #[derive(Debug, PartialEq, TryFrom)]
        pub struct Amount(u32);

        assert_eq!(Amount::try_from(99), Err(AmountError::TooSmall));
        assert_eq!(Amount::try_from(1_001), Err(AmountError::TooBig));

        assert_eq!(Amount::try_from(100).unwrap().into_inner(), 100);
    }
}
