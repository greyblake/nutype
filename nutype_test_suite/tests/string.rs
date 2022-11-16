use nutype::nutype;

#[cfg(test)]
mod sanitizers {
    use super::*;

    #[test]
    fn test_trim() {
        #[nutype(sanitize(trim))]
        pub struct Name(String);

        assert_eq!(Name::new("").into_inner(), "");
        assert_eq!(Name::new("\n\t ").into_inner(), "");
        assert_eq!(Name::new(" ! ").into_inner(), "!");
        assert_eq!(Name::new(" foo \n bar ").into_inner(), "foo \n bar");
    }

    #[test]
    fn test_lowercase() {
        #[nutype(sanitize(lowercase))]
        pub struct Name(String);

        assert_eq!(Name::new("  ").into_inner(), "  ");
        assert_eq!(Name::new("Hello THERE").into_inner(), "hello there");
    }

    #[test]
    fn test_uppercase() {
        #[nutype(sanitize(uppercase))]
        pub struct Name(String);

        assert_eq!(Name::new(" ").into_inner(), " ");
        assert_eq!(Name::new("Hello THERE").into_inner(), "HELLO THERE");
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(sanitize(with = |s: String| s.trim().to_uppercase() ))]
            pub struct Name(String);

            assert_eq!(Name::new(" Anton\n\n").into_inner(), "ANTON");
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(sanitize(with = |s| s.trim().to_uppercase() ))]
            pub struct Name(String);

            assert_eq!(Name::new(" Anton\n\n").into_inner(), "ANTON");
        }

        fn sanitize_name(raw_name: String) -> String {
            raw_name.trim().to_uppercase()
        }

        #[test]
        fn test_with_function() {
            #[nutype(sanitize(with = sanitize_name))]
            pub struct Name(String);

            assert_eq!(Name::new(" Anton\n\n").into_inner(), "ANTON");
        }
    }

    #[test]
    fn test_many_sanitizers() {
        #[nutype(sanitize(trim, uppercase, with = |s| s[1..=2].to_string()))]
        pub struct Country(String);

        assert_eq!(Country::new(" Deutschland ").into_inner(), "EU");
    }

    #[test]
    fn test_from_trait() {
        #[nutype(sanitize(trim, lowercase))]
        pub struct Email(String);

        assert_eq!(
            Email::from("  Email@example.com ").into_inner(),
            "email@example.com"
        );
    }
}

#[cfg(test)]
mod validators {
    use super::*;

    #[test]
    fn test_max_len() {
        #[nutype(validate(max_len = 5))]
        pub struct Name(String);

        assert_eq!(Name::new("Anton").unwrap().into_inner(), "Anton");
        assert_eq!(Name::new("Serhii"), Err(NameError::TooLong));
    }

    #[test]
    fn test_min_len() {
        #[nutype(validate(min_len = 6))]
        pub struct Name(String);

        assert_eq!(Name::new("Anton"), Err(NameError::TooShort));
        assert_eq!(Name::new("Serhii").unwrap().into_inner(), "Serhii");
    }

    #[test]
    fn test_present() {
        #[nutype(validate(present))]
        pub struct Name(String);

        assert_eq!(Name::new(""), Err(NameError::Missing));
        assert_eq!(Name::new(" ").unwrap().into_inner(), " ");
        assert_eq!(Name::new("Julia").unwrap().into_inner(), "Julia");
    }

    #[test]
    fn test_many_validators() {
        #[nutype(validate(min_len = 3, max_len = 6))]
        pub struct Name(String);

        assert_eq!(Name::new("Jo"), Err(NameError::TooShort));
        assert_eq!(Name::new("Friedrich"), Err(NameError::TooLong));
        assert_eq!(Name::new("Julia").unwrap().into_inner(), "Julia");
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(validate(with = |e: &str| e.contains('@')))]
            pub struct Email(String);

            assert_eq!(Email::new("foo.bar.example"), Err(EmailError::Invalid));
            assert_eq!(
                Email::new("foo@bar.example").unwrap().into_inner(),
                "foo@bar.example"
            );
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(validate(with = |e| e.contains('@')))]
            pub struct Email(String);

            assert_eq!(Email::new("foo.bar.example"), Err(EmailError::Invalid));
            assert_eq!(
                Email::new("foo@bar.example").unwrap().into_inner(),
                "foo@bar.example"
            );
        }

        fn validate_email(val: &str) -> bool {
            val.contains('@')
        }

        #[test]
        fn test_with_function() {
            #[nutype(validate(with = validate_email))]
            pub struct Email(String);

            assert_eq!(Email::new("foo.bar.example"), Err(EmailError::Invalid));
            assert_eq!(
                Email::new("foo@bar.example").unwrap().into_inner(),
                "foo@bar.example"
            );
        }
    }

    #[test]
    fn test_try_from_trait() {
        #[nutype(validate(present))]
        pub struct Name(String);

        assert_eq!(Name::try_from(""), Err(NameError::Missing));
        assert_eq!(Name::try_from("Tom").unwrap().into_inner(), "Tom");
    }
}

#[cfg(test)]
mod complex {
    use super::*;

    #[test]
    fn test_sanitizers_and_validators() {
        /// Some documentation for Name
        /// goes here.
        #[nutype(
            sanitize(trim, with = |s| s.to_uppercase())
            validate(present, max_len = 6)
        )]
        pub struct Name(String);

        assert_eq!(Name::new("    "), Err(NameError::Missing));
        assert_eq!(Name::new("Willy Brandt"), Err(NameError::TooLong));
        assert_eq!(Name::new("   Brandt  ").unwrap().into_inner(), "BRANDT");
    }
}

#[cfg(test)]
mod visibility {
    mod encapsulated {
        use nutype::nutype;

        #[nutype(sanitize(lowercase))]
        pub struct Email(String);
    }

    #[test]
    fn test_public_visibility() {
        let email = encapsulated::Email::new("FOO@bar.com");
        assert_eq!(email.into_inner(), "foo@bar.com");
    }
}
