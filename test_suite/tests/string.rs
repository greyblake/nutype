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
        #[nutype(sanitize(trim, lowercase), derive(Debug, PartialEq, From))]
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
    fn test_len_char_max() {
        #[nutype(
            validate(len_char_max = 5),
            derive(TryFrom, Debug, Clone, PartialEq, PartialOrd, FromStr, AsRef)
        )]
        pub struct Name(String);

        assert_eq!(Name::try_new("Anton").unwrap().into_inner(), "Anton");
        assert_eq!(Name::try_new("Serhii"), Err(NameError::LenCharMaxViolated));

        // Ukrainian, Cyrillic. Every char is 2 bytes.
        assert_eq!(Name::try_new("Антон").unwrap().into_inner(), "Антон");
    }

    #[test]
    fn test_len_char_min() {
        #[nutype(validate(len_char_min = 6), derive(Debug, PartialEq))]
        pub struct Name(String);

        assert_eq!(Name::try_new("Anton"), Err(NameError::LenCharMinViolated));
        assert_eq!(Name::try_new("Serhii").unwrap().into_inner(), "Serhii");

        // Ukrainian, Cyrillic. Every char is 2 bytes.
        assert_eq!(Name::try_new("Антон"), Err(NameError::LenCharMinViolated));
    }

    #[test]
    fn test_not_empty() {
        #[nutype(validate(not_empty), derive(Debug, PartialEq))]
        pub struct Name(String);

        assert_eq!(Name::try_new(""), Err(NameError::NotEmptyViolated));
        assert_eq!(Name::try_new(" ").unwrap().into_inner(), " ");
        assert_eq!(Name::try_new("Julia").unwrap().into_inner(), "Julia");
    }

    #[test]
    fn test_many_validators() {
        #[nutype(validate(len_char_min = 3, len_char_max = 6), derive(Debug, PartialEq))]
        pub struct Name(String);

        assert_eq!(Name::try_new("Jo"), Err(NameError::LenCharMinViolated));
        assert_eq!(
            Name::try_new("Friedrich"),
            Err(NameError::LenCharMaxViolated)
        );
        assert_eq!(Name::try_new("Julia").unwrap().into_inner(), "Julia");
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(validate(predicate = |e: &str| e.contains('@')), derive(Debug, PartialEq))]
            pub struct Email(String);

            assert_eq!(
                Email::try_new("foo.bar.example"),
                Err(EmailError::PredicateViolated)
            );
            assert_eq!(
                Email::try_new("foo@bar.example").unwrap().into_inner(),
                "foo@bar.example"
            );
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(validate(predicate = |e| e.contains('@')), derive(Debug, PartialEq))]
            pub struct Email(String);

            assert_eq!(
                Email::try_new("foo.bar.example"),
                Err(EmailError::PredicateViolated)
            );
            assert_eq!(
                Email::try_new("foo@bar.example").unwrap().into_inner(),
                "foo@bar.example"
            );
        }

        fn validate_email(val: &str) -> bool {
            val.contains('@')
        }

        #[test]
        fn test_with_function() {
            #[nutype(validate(predicate = validate_email), derive(Debug, PartialEq))]
            pub struct Email(String);

            assert_eq!(
                Email::try_new("foo.bar.example"),
                Err(EmailError::PredicateViolated)
            );
            assert_eq!(
                Email::try_new("foo@bar.example").unwrap().into_inner(),
                "foo@bar.example"
            );
        }
    }

    #[test]
    fn test_try_from_trait() {
        #[nutype(validate(not_empty), derive(Debug, PartialEq, TryFrom))]
        pub struct Name(String);

        assert_eq!(Name::try_from(""), Err(NameError::NotEmptyViolated));
        assert_eq!(Name::try_from("Tom").unwrap().into_inner(), "Tom");
    }

    #[test]
    fn test_try_from_trait_without_validation() {
        #[nutype(derive(Debug, PartialEq, TryFrom))]
        pub struct Name(String);

        assert_eq!(Name::try_from("Tom").unwrap().into_inner(), "Tom");
    }

    #[test]
    fn test_error() {
        fn ensure_type_implements_error<T: core::error::Error>() {}

        #[nutype(validate(not_empty), derive(Debug, PartialEq))]
        pub struct Email(String);

        ensure_type_implements_error::<EmailError>();
    }

    #[test]
    fn test_error_display() {
        #[nutype(validate(not_empty))]
        pub struct Email(String);

        assert_eq!(EmailError::NotEmptyViolated.to_string(), "Email is empty.");
    }

    #[test]
    fn test_error_display_utf16() {
        #[nutype(validate(len_utf16_min = 2, len_utf16_max = 5))]
        pub struct JsString(String);

        assert_eq!(
            JsStringError::LenUtf16MinViolated.to_string(),
            "JsString is too short: the minimum valid UTF-16 length is 2 code units."
        );
        assert_eq!(
            JsStringError::LenUtf16MaxViolated.to_string(),
            "JsString is too long: the maximum valid UTF-16 length is 5 code units."
        );
    }

    #[test]
    fn test_error_display_utf16_singular() {
        #[nutype(validate(len_utf16_min = 1, len_utf16_max = 1))]
        pub struct SingleUnit(String);

        assert_eq!(
            SingleUnitError::LenUtf16MinViolated.to_string(),
            "SingleUnit is too short: the minimum valid UTF-16 length is 1 code unit."
        );
        assert_eq!(
            SingleUnitError::LenUtf16MaxViolated.to_string(),
            "SingleUnit is too long: the maximum valid UTF-16 length is 1 code unit."
        );
    }

    mod when_boundaries_defined_as_constants {
        use super::*;

        const MIN_LEN: usize = 3;
        const MAX_LEN: usize = 10;

        #[nutype(validate(len_char_min = MIN_LEN, len_char_max = MAX_LEN), derive(Debug))]
        struct Login(String);

        #[test]
        fn test_boundaries_defined_as_constants() {
            assert_eq!(
                Login::try_new("ab").unwrap_err(),
                LoginError::LenCharMinViolated,
            );
            assert_eq!(
                Login::try_new("abc").unwrap().into_inner(),
                "abc".to_string(),
            );

            assert_eq!(
                Login::try_new("abcdefghijk").unwrap_err(),
                LoginError::LenCharMaxViolated,
            );
            assert_eq!(
                Login::try_new("abcdefghij").unwrap().into_inner(),
                "abcdefghij".to_string(),
            );
        }
    }

    #[test]
    fn test_len_utf16_max() {
        // UTF-16 length validation - useful for JavaScript interop
        // 🦀 (crab emoji) is 1 char but 2 UTF-16 code units
        #[nutype(validate(len_utf16_max = 5), derive(Debug, PartialEq))]
        pub struct ShortText(String);

        // ASCII: 5 chars = 5 UTF-16 code units (valid)
        assert_eq!(ShortText::try_new("Anton").unwrap().into_inner(), "Anton");

        // ASCII: 6 chars = 6 UTF-16 code units (invalid)
        assert_eq!(
            ShortText::try_new("Serhii"),
            Err(ShortTextError::LenUtf16MaxViolated)
        );

        // Emoji: 2 crabs = 2 chars but 4 UTF-16 code units (valid)
        assert_eq!(ShortText::try_new("🦀🦀").unwrap().into_inner(), "🦀🦀");

        // Emoji: 3 crabs = 3 chars but 6 UTF-16 code units (invalid)
        assert_eq!(
            ShortText::try_new("🦀🦀🦀"),
            Err(ShortTextError::LenUtf16MaxViolated)
        );
    }

    #[test]
    fn test_len_utf16_min() {
        // UTF-16 length validation - useful for JavaScript interop
        #[nutype(validate(len_utf16_min = 4), derive(Debug, PartialEq))]
        pub struct LongText(String);

        // ASCII: 3 chars = 3 UTF-16 code units (invalid)
        assert_eq!(
            LongText::try_new("abc"),
            Err(LongTextError::LenUtf16MinViolated)
        );

        // ASCII: 4 chars = 4 UTF-16 code units (valid)
        assert_eq!(LongText::try_new("abcd").unwrap().into_inner(), "abcd");

        // Emoji: 1 crab = 1 char but 2 UTF-16 code units (invalid)
        assert_eq!(
            LongText::try_new("🦀"),
            Err(LongTextError::LenUtf16MinViolated)
        );

        // Emoji: 2 crabs = 2 chars but 4 UTF-16 code units (valid)
        assert_eq!(LongText::try_new("🦀🦀").unwrap().into_inner(), "🦀🦀");
    }

    #[test]
    fn test_len_utf16_min_and_max() {
        // Combined UTF-16 length validation
        #[nutype(
            validate(len_utf16_min = 2, len_utf16_max = 4),
            derive(Debug, PartialEq)
        )]
        pub struct BoundedText(String);

        // Too short: 1 UTF-16 code unit
        assert_eq!(
            BoundedText::try_new("a"),
            Err(BoundedTextError::LenUtf16MinViolated)
        );

        // Just right: 2 UTF-16 code units
        assert_eq!(BoundedText::try_new("ab").unwrap().into_inner(), "ab");

        // Just right: 4 UTF-16 code units
        assert_eq!(BoundedText::try_new("abcd").unwrap().into_inner(), "abcd");

        // Too long: 5 UTF-16 code units
        assert_eq!(
            BoundedText::try_new("abcde"),
            Err(BoundedTextError::LenUtf16MaxViolated)
        );

        // 1 emoji = 2 UTF-16 code units (valid)
        assert_eq!(BoundedText::try_new("🦀").unwrap().into_inner(), "🦀");

        // 2 emojis = 4 UTF-16 code units (valid)
        assert_eq!(BoundedText::try_new("🦀🦀").unwrap().into_inner(), "🦀🦀");

        // 3 emojis = 6 UTF-16 code units (invalid)
        assert_eq!(
            BoundedText::try_new("🦀🦀🦀"),
            Err(BoundedTextError::LenUtf16MaxViolated)
        );
    }

    mod when_utf16_boundaries_defined_as_constants {
        use super::*;

        const MIN_UTF16_LEN: usize = 2;
        const MAX_UTF16_LEN: usize = 6;

        #[nutype(validate(len_utf16_min = MIN_UTF16_LEN, len_utf16_max = MAX_UTF16_LEN), derive(Debug))]
        struct JsText(String);

        #[test]
        fn test_utf16_boundaries_defined_as_constants() {
            assert_eq!(
                JsText::try_new("a").unwrap_err(),
                JsTextError::LenUtf16MinViolated,
            );
            assert_eq!(
                JsText::try_new("ab").unwrap().into_inner(),
                "ab".to_string(),
            );

            assert_eq!(
                JsText::try_new("abcdefg").unwrap_err(),
                JsTextError::LenUtf16MaxViolated,
            );
            assert_eq!(
                JsText::try_new("abcdef").unwrap().into_inner(),
                "abcdef".to_string(),
            );

            // Test with emoji (2 UTF-16 code units each)
            assert_eq!(
                JsText::try_new("🦀🦀🦀🦀").unwrap_err(), // 8 UTF-16 code units
                JsTextError::LenUtf16MaxViolated,
            );
            assert_eq!(
                JsText::try_new("🦀🦀🦀").unwrap().into_inner(), // 6 UTF-16 code units
                "🦀🦀🦀".to_string(),
            );
        }
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
            sanitize(trim, with = |s| s.to_uppercase()),
            validate(not_empty, len_char_max = 6),
            derive(Debug, PartialEq)
        )]
        pub struct Name(String);

        assert_eq!(Name::try_new("    "), Err(NameError::NotEmptyViolated));
        assert_eq!(
            Name::try_new("Willy Brandt"),
            Err(NameError::LenCharMaxViolated)
        );
        assert_eq!(Name::try_new("   Brandt  ").unwrap().into_inner(), "BRANDT");
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
        #[nutype(derive(Debug, Hash, From, FromStr, Borrow, Clone))]
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
    fn test_with_validation() {
        #[nutype(
            validate(not_empty),
            derive(Debug, Hash, TryFrom, FromStr, Borrow, Clone)
        )]
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
        #[nutype(sanitize(trim), derive(Into))]
        pub struct Name(String);

        let name = Name::new("  Anna");
        let name: String = name.into();
        assert_eq!(name, "Anna")
    }

    #[test]
    fn test_trait_from_str() {
        #[nutype(derive(From))]
        pub struct Name(String);

        let name = Name::from("Anna");
        assert_eq!(name.into_inner(), "Anna")
    }

    #[test]
    fn test_trait_from_string() {
        #[nutype(derive(From))]
        pub struct Name(String);

        let name = Name::from("Anna".to_string());
        assert_eq!(name.into_inner(), "Anna")
    }

    #[test]
    fn test_trait_as_ref() {
        #[nutype(derive(AsRef))]
        pub struct Name(String);

        let name = Name::new("Anna");
        let name_ref: &str = name.as_ref();
        assert_eq!(name_ref, "Anna")
    }

    #[test]
    fn test_trait_deref() {
        #[nutype(derive(Deref))]
        pub struct Name(String);

        let name = Name::new("Anna");

        // Let's do something with deref-coercion:
        assert_eq!(name.len(), 4);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_trait_borrow_str() {
        use core::borrow::Borrow;

        #[nutype(derive(Borrow))]
        pub struct Name(String);

        let name = Name::new("Anna");
        let name_borrowed: &str = name.borrow();
        assert_eq!(name_borrowed, "Anna");
    }

    #[test]
    fn test_trait_borrow_string() {
        use core::borrow::Borrow;

        #[nutype(derive(Borrow))]
        pub struct Name(String);

        let name = Name::new("Anna");
        let name_borrowed: &String = name.borrow();
        assert_eq!(name_borrowed, "Anna");
    }

    #[test]
    fn test_trait_try_from_str() {
        #[nutype(validate(not_empty), derive(Debug, TryFrom))]
        pub struct Name(String);

        let name = Name::try_from("Anna").unwrap();
        assert_eq!(name.into_inner(), "Anna");

        let error = Name::try_from("").unwrap_err();
        assert_eq!(error, NameError::NotEmptyViolated);
    }

    #[test]
    fn test_trait_try_from_string() {
        #[nutype(validate(not_empty), derive(Debug, TryFrom))]
        pub struct Name(String);

        let name = Name::try_from("Anna".to_string()).unwrap();
        assert_eq!(name.into_inner(), "Anna");

        let error = Name::try_from("".to_string()).unwrap_err();
        assert_eq!(error, NameError::NotEmptyViolated);
    }

    #[test]
    fn test_trait_display() {
        #[nutype(derive(Display))]
        pub struct Name(String);

        let name = Name::new("Serhii");
        assert_eq!(name.to_string(), "Serhii");
    }

    #[cfg(test)]
    mod trait_default {
        use super::*;

        #[test]
        fn test_default_without_validation() {
            #[nutype(default = "Anonymous", derive(Default))]
            pub struct Name(String);

            assert_eq!(Name::default().into_inner(), "Anonymous");
        }

        #[test]
        fn test_default_with_validation_when_valid() {
            #[nutype(validate(len_char_min = 5), default = "Anonymous", derive(Default))]
            pub struct Name(String);

            assert_eq!(Name::default().into_inner(), "Anonymous");
        }

        #[test]
        #[should_panic(expected = "Default value for type `Name` is invalid")]
        fn test_default_with_validation_when_invalid() {
            #[nutype(validate(len_char_min = 5), default = "Nope", derive(Default))]
            pub struct Name(String);

            Name::default();
        }
    }

    #[cfg(feature = "serde")]
    mod serialization {
        use super::*;

        mod json_format {
            use super::*;

            #[test]
            fn test_trait_serialize() {
                #[nutype(derive(Serialize))]
                pub struct Email(String);

                let email = Email::new("my@example.com");
                let email_json = serde_json::to_string(&email).unwrap();
                assert_eq!(email_json, "\"my@example.com\"");
            }

            #[test]
            fn test_trait_deserialize_without_validation() {
                #[nutype(derive(Deserialize))]
                pub struct NaiveEmail(String);

                {
                    let email: NaiveEmail = serde_json::from_str("\"foobar\"").unwrap();
                    assert_eq!(email.into_inner(), "foobar");
                }
            }

            #[test]
            fn test_trait_deserialize_with_validation() {
                #[nutype(
                    validate(predicate = |address| address.contains('@') ),
                    derive(Deserialize),
                )]
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

        mod ron_format {
            use super::*;

            #[test]
            fn test_ron_roundtrip() {
                #[nutype(derive(Serialize, Deserialize, PartialEq, Debug))]
                pub struct Email(String);

                let email = Email::new("me@example.com");

                let serialized = ron::to_string(&email).unwrap();
                let deserialized: Email = ron::from_str(&serialized).unwrap();

                assert_eq!(deserialized, email);
            }
        }

        mod message_pack_format {
            use super::*;

            #[test]
            fn test_rmp_roundtrip() {
                #[nutype(derive(Serialize, Deserialize, PartialEq, Debug))]
                pub struct Email(String);

                let email = Email::new("me@example.com");

                let bytes = rmp_serde::to_vec(&email).unwrap();
                let deserialized: Email = rmp_serde::from_slice(&bytes).unwrap();

                assert_eq!(deserialized, email);
            }
        }
    }
}

#[cfg(feature = "new_unchecked")]
mod new_unchecked {
    use super::*;

    #[test]
    fn test_new_unchecked() {
        #[nutype(new_unchecked, sanitize(trim), validate(len_char_min = 8))]
        pub struct Name(String);

        let name = unsafe { Name::new_unchecked(" boo ".to_string()) };
        assert_eq!(name.into_inner(), " boo ");
    }
}

#[cfg(test)]
#[cfg(feature = "schemars08")]
mod derive_schemars_json_schema {
    use super::*;
    use schemars::{schema_for, JsonSchema};

    #[test]
    fn test_json_schema_derive() {
        #[nutype(derive(JsonSchema))]
        pub struct CustomerIdentifier(String);

        assert_eq!(CustomerIdentifier::schema_name(), "CustomerIdentifier");
        // Make sure it compiles
        let _schema = schema_for!(CustomerIdentifier);
    }
}

#[cfg(test)]
#[cfg(feature = "regex")]
mod validation_with_regex {
    use super::*;
    use std::sync::LazyLock;

    use lazy_static::lazy_static;
    use once_cell::sync::Lazy;
    use regex::Regex;

    static PHONE_REGEX_LAZY_LOCK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^[0-9]{3}-[0-9]{3}$").unwrap());

    lazy_static! {
        static ref PHONE_REGEX_LAZY_STATIC: Regex = Regex::new("^[0-9]{3}-[0-9]{3}$").unwrap();
    }

    static PHONE_REGEX_ONCE_CELL: Lazy<Regex> =
        Lazy::new(|| Regex::new("[0-9]{3}-[0-9]{3}$").unwrap());

    #[test]
    fn test_regex_as_string() {
        #[nutype(validate(regex = "^[0-9]{3}-[0-9]{3}$"), derive(Debug, PartialEq))]
        pub struct PhoneNumber(String);

        // PredicateViolated
        assert_eq!(
            PhoneNumber::try_new("123456"),
            Err(PhoneNumberError::RegexViolated)
        );

        // Valid
        let inner = PhoneNumber::try_new("123-456").unwrap().into_inner();
        assert_eq!(inner, "123-456".to_string());
    }

    #[test]
    fn test_regex_with_lazy_lock() {
        #[nutype(validate(regex = PHONE_REGEX_LAZY_LOCK), derive(Debug, PartialEq))]
        pub struct PhoneNumber(String);

        // PredicateViolated
        assert_eq!(
            PhoneNumber::try_new("123456"),
            Err(PhoneNumberError::RegexViolated)
        );

        // Valid
        let inner = PhoneNumber::try_new("123-456").unwrap().into_inner();
        assert_eq!(inner, "123-456".to_string());
    }

    #[test]
    fn test_regex_with_lazy_static() {
        #[nutype(validate(regex = PHONE_REGEX_LAZY_STATIC), derive(Debug, PartialEq))]
        pub struct PhoneNumber(String);

        // PredicateViolated
        assert_eq!(
            PhoneNumber::try_new("123456"),
            Err(PhoneNumberError::RegexViolated)
        );

        // Valid
        let inner = PhoneNumber::try_new("123-456").unwrap().into_inner();
        assert_eq!(inner, "123-456".to_string());
    }

    #[test]
    fn test_regex_with_once_cell_lazy() {
        #[nutype(validate(regex = PHONE_REGEX_ONCE_CELL), derive(Debug, PartialEq))]
        pub struct PhoneNumber(String);

        // PredicateViolated
        assert_eq!(
            PhoneNumber::try_new("123456"),
            Err(PhoneNumberError::RegexViolated)
        );

        // Valid
        let inner = PhoneNumber::try_new("123-456").unwrap().into_inner();
        assert_eq!(inner, "123-456".to_string());
    }
}

#[cfg(test)]
#[cfg(feature = "valuable")]
mod derive_valuable {
    use super::*;
    use valuable::Valuable;

    #[test]
    fn test_valuable_derive() {
        #[nutype(derive(Valuable))]
        pub struct Name(String);

        assert_eq!(
            format!("{:?}", Name::new("Sherlock".to_owned()).as_value()),
            r#"Name("Sherlock")"#
        );
    }
}

mod custom_error {
    use super::*;
    use thiserror::Error;

    #[nutype(
        validate(with = validate_name, error = NameError),
        derive(Debug, AsRef, PartialEq),
    )]
    struct Name(String);

    fn validate_name(name: &str) -> Result<(), NameError> {
        if name.len() < 3 {
            Err(NameError::TooShort)
        } else if name.len() > 10 {
            Err(NameError::TooLong)
        } else {
            Ok(())
        }
    }

    #[derive(Error, Debug, PartialEq)]
    enum NameError {
        #[error("Name is too short.")]
        TooShort,

        #[error("Name is too long.")]
        TooLong,
    }

    #[test]
    fn test_custom_error() {
        assert_eq!(
            Name::try_new("JohnJohnJohnJohnJohn"),
            Err(NameError::TooLong)
        );

        assert_eq!(Name::try_new("Jo"), Err(NameError::TooShort));

        let name = Name::try_new("John").unwrap();
        assert_eq!(name.as_ref(), "John");
    }
}

#[cfg(test)]
mod constructor_visibility {
    use super::*;

    // Note: UI tests verify that `constructor(visibility = private)` makes constructors
    // inaccessible from outside the defining module.

    mod private_visibility {
        use super::*;

        #[nutype(
            sanitize(trim),
            constructor(visibility = private),
            derive(Debug, AsRef),
        )]
        pub struct PrivateName(String);

        // Factory function within the same module can access the private constructor
        pub fn create_name(s: &str) -> PrivateName {
            PrivateName::new(s)
        }

        #[test]
        fn test_private_new_accessible_within_module() {
            let name = PrivateName::new("  test  ");
            assert_eq!(name.as_ref(), "test");
        }
    }

    #[test]
    fn test_private_via_factory() {
        // Access via factory function works
        let name = private_visibility::create_name("  hello  ");
        assert_eq!(name.as_ref(), "hello");
    }

    mod private_try_new_visibility {
        use super::*;

        #[nutype(
            validate(not_empty),
            constructor(visibility = private),
            derive(Debug, AsRef),
        )]
        pub struct PrivateValidatedName(String);

        pub fn create_validated_name(
            s: &str,
        ) -> Result<PrivateValidatedName, PrivateValidatedNameError> {
            PrivateValidatedName::try_new(s)
        }

        #[test]
        fn test_private_try_new_accessible_within_module() {
            let name = PrivateValidatedName::try_new("test").unwrap();
            assert_eq!(name.as_ref(), "test");
        }
    }

    #[test]
    fn test_private_try_new_via_factory() {
        let name = private_try_new_visibility::create_validated_name("world").unwrap();
        assert_eq!(name.as_ref(), "world");
    }

    mod pub_crate_visibility {
        use super::*;

        #[nutype(
            sanitize(trim),
            constructor(visibility = pub(crate)),
            derive(Debug, AsRef),
        )]
        pub struct CrateName(String);

        #[test]
        fn test_pub_crate_new() {
            let name = CrateName::new("  crate  ");
            assert_eq!(name.as_ref(), "crate");
        }
    }

    mod pub_crate_try_new_visibility {
        use super::*;

        #[nutype(
            validate(not_empty),
            constructor(visibility = pub(crate)),
            derive(Debug, AsRef),
        )]
        pub struct CrateValidatedName(String);

        #[test]
        fn test_pub_crate_try_new() {
            let name = CrateValidatedName::try_new("crate").unwrap();
            assert_eq!(name.as_ref(), "crate");
        }
    }

    mod explicit_pub_visibility {
        use super::*;

        #[nutype(
            sanitize(trim),
            constructor(visibility = pub),
            derive(Debug, AsRef),
        )]
        pub struct ExplicitPubName(String);

        #[test]
        fn test_explicit_pub_visibility() {
            // Explicit pub should work the same as default
            let name = ExplicitPubName::new("  explicit  ");
            assert_eq!(name.as_ref(), "explicit");
        }
    }

    mod explicit_pub_try_new_visibility {
        use super::*;

        #[nutype(
            validate(not_empty),
            constructor(visibility = pub),
            derive(Debug, AsRef),
        )]
        pub struct ExplicitPubValidatedName(String);

        #[test]
        fn test_explicit_pub_try_new_visibility() {
            // Explicit pub should work the same as default
            let name = ExplicitPubValidatedName::try_new("test").unwrap();
            assert_eq!(name.as_ref(), "test");
        }
    }
}
