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
        #[derive(Debug, PartialEq, From)]
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
        #[derive(*)]
        pub struct Name(String);

        assert_eq!(Name::new("Anton").unwrap().into_inner(), "Anton");
        assert_eq!(Name::new("Serhii"), Err(NameError::TooLong));
    }

    #[test]
    fn test_min_len() {
        #[nutype(validate(min_len = 6))]
        #[derive(Debug, PartialEq)]
        pub struct Name(String);

        assert_eq!(Name::new("Anton"), Err(NameError::TooShort));
        assert_eq!(Name::new("Serhii").unwrap().into_inner(), "Serhii");
    }

    #[test]
    fn test_not_empty() {
        #[nutype(validate(not_empty))]
        #[derive(Debug, PartialEq)]
        pub struct Name(String);

        assert_eq!(Name::new(""), Err(NameError::Empty));
        assert_eq!(Name::new(" ").unwrap().into_inner(), " ");
        assert_eq!(Name::new("Julia").unwrap().into_inner(), "Julia");
    }

    #[test]
    fn test_many_validators() {
        #[nutype(validate(min_len = 3, max_len = 6))]
        #[derive(Debug, PartialEq)]
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
            #[derive(Debug, PartialEq)]
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
            #[derive(Debug, PartialEq)]
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
            #[derive(Debug, PartialEq)]
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
        #[nutype(validate(not_empty))]
        #[derive(Debug, PartialEq, TryFrom)]
        pub struct Name(String);

        assert_eq!(Name::try_from(""), Err(NameError::Empty));
        assert_eq!(Name::try_from("Tom").unwrap().into_inner(), "Tom");
    }

    #[test]
    fn test_error() {
        fn ensure_type_implements_error<T: std::error::Error>() {}

        #[nutype(validate(not_empty))]
        #[derive(Debug, PartialEq)]
        pub struct Email(String);

        ensure_type_implements_error::<EmailError>();
    }

    #[test]
    fn test_error_display() {
        #[nutype(validate(not_empty))]
        pub struct Email(String);

        assert_eq!(EmailError::Empty.to_string(), "empty");
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
            validate(not_empty, max_len = 6)
        )]
        #[derive(Debug, PartialEq)]
        pub struct Name(String);

        assert_eq!(Name::new("    "), Err(NameError::Empty));
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

#[cfg(test)]
mod derives {
    use super::*;
    use test_suite::test_helpers::traits::*;

    #[test]
    fn test_without_validation() {
        #[nutype]
        #[derive(Debug, Hash, From, FromStr, Borrow, Clone)]
        pub struct Name(String);

        should_implement_hash::<Name>();
        should_implement_debug::<Name>();
        should_implement_from::<Name, String>();
        should_implement_from::<Name, &str>();
        should_implement_from_str::<Name>();
        should_implement_borrow::<Name, str>();
        should_implement_borrow::<Name, String>();
        should_implement_clone::<Name>();
    }

    #[test]
    fn test_with_validaiton() {
        #[nutype(validate(not_empty))]
        #[derive(Debug, Hash, TryFrom, FromStr, Borrow, Clone)]
        pub struct Name(String);

        should_implement_hash::<Name>();
        should_implement_debug::<Name>();
        should_implement_try_from::<Name, String>();
        should_implement_try_from::<Name, &str>();
        should_implement_from_str::<Name>();
        should_implement_borrow::<Name, str>();
        should_implement_borrow::<Name, String>();
        should_implement_clone::<Name>();
    }

    #[test]
    fn test_trait_into() {
        #[nutype(sanitize(trim))]
        #[derive(Into)]
        pub struct Name(String);

        let name = Name::new("  Anna");
        let name: String = name.into();
        assert_eq!(name, "Anna")
    }

    #[test]
    fn test_trait_from_str() {
        #[nutype]
        #[derive(From)]
        pub struct Name(String);

        let name = Name::from("Anna");
        assert_eq!(name.into_inner(), "Anna")
    }

    #[test]
    fn test_trait_from_string() {
        #[nutype]
        #[derive(From)]
        pub struct Name(String);

        let name = Name::from("Anna".to_string());
        assert_eq!(name.into_inner(), "Anna")
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype]
        #[derive(AsRef)]
        pub struct Name(String);

        let name = Name::new("Anna");
        let name_ref: &str = name.as_ref();
        assert_eq!(name_ref, "Anna")
    }

    #[test]
    fn test_trait_borrow_str() {
        use std::borrow::Borrow;

        #[nutype]
        #[derive(Borrow)]
        pub struct Name(String);

        let name = Name::new("Anna");
        let name_borrowed: &str = name.borrow();
        assert_eq!(name_borrowed, "Anna");
    }

    #[test]
    fn test_trait_borrow_string() {
        use std::borrow::Borrow;

        #[nutype]
        #[derive(Borrow)]
        pub struct Name(String);

        let name = Name::new("Anna");
        let name_borrowed: &String = name.borrow();
        assert_eq!(name_borrowed, "Anna");
    }

    #[test]
    fn test_trait_try_from_str() {
        #[nutype(validate(not_empty))]
        #[derive(Debug, TryFrom)]
        pub struct Name(String);

        let name = Name::try_from("Anna").unwrap();
        assert_eq!(name.into_inner(), "Anna");

        let error = Name::try_from("").unwrap_err();
        assert_eq!(error, NameError::Empty);
    }

    #[test]
    fn test_trait_try_from_string() {
        #[nutype(validate(not_empty))]
        #[derive(Debug, TryFrom)]
        pub struct Name(String);

        let name = Name::try_from("Anna".to_string()).unwrap();
        assert_eq!(name.into_inner(), "Anna");

        let error = Name::try_from("".to_string()).unwrap_err();
        assert_eq!(error, NameError::Empty);
    }

    #[test]
    fn test_trait_display() {
        #[nutype]
        #[derive(Display)]
        pub struct Name(String);

        let name = Name::new("Serhii");
        assert_eq!(name.to_string(), "Serhii");
    }

    #[cfg(feature = "serde1")]
    #[test]
    fn test_trait_serialize() {
        #[nutype]
        #[derive(Serialize)]
        pub struct Email(String);

        let email = Email::new("my@example.com");
        let email_json = serde_json::to_string(&email).unwrap();
        assert_eq!(email_json, "\"my@example.com\"");
    }

    #[cfg(feature = "serde1")]
    #[test]
    fn test_trait_deserialize_without_validation() {
        #[nutype]
        #[derive(Deserialize)]
        pub struct NaiveEmail(String);

        {
            let email: NaiveEmail = serde_json::from_str("\"foobar\"").unwrap();
            assert_eq!(email.into_inner(), "foobar");
        }
    }

    #[cfg(feature = "serde1")]
    #[test]
    fn test_trait_deserialize_with_validation() {
        #[nutype(
            validate(with = |address| address.contains('@') )
        )]
        #[derive(Deserialize)]
        pub struct NaiveEmail(String);

        {
            let res: Result<NaiveEmail, _> = serde_json::from_str("\"foobar\"");
            assert!(res.is_err());
        }

        {
            let email: NaiveEmail = serde_json::from_str("\"foo@bar.com\"").unwrap();
            assert_eq!(email.into_inner(), "foo@bar.com");
        }
    }
}
