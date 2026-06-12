//! Tests for attribute passthrough (issue #229 and friends):
//! * struct-level attributes are forwarded onto the generated struct;
//! * field-level attributes are forwarded onto the inner field;
//! * `#[serde(...)]` field attributes (`with`, `serialize_with`,
//!   `deserialize_with`) and struct-level `#[serde(transparent)]` are consumed
//!   by nutype and woven into its generated serde impls, with sanitization and
//!   validation still running on deserialization.

use nutype::nutype;

// ---------------------------------------------------------------------------
// Struct-level attribute forwarding
// ---------------------------------------------------------------------------
mod struct_attr_forwarding {
    use super::*;

    #[test]
    fn repr_transparent_is_forwarded() {
        #[nutype(derive(Debug, PartialEq))]
        #[repr(transparent)]
        struct Amount(i32);

        assert_eq!(Amount::new(5), Amount::new(5));
    }

    #[test]
    fn lint_attr_is_forwarded() {
        #[nutype(derive(Debug))]
        #[allow(missing_docs)]
        struct Tag(String);

        assert_eq!(Tag::new("x").into_inner(), "x");
    }

    #[test]
    fn cfg_attr_wrapped_attr_is_forwarded() {
        #[nutype(derive(Debug))]
        #[cfg_attr(test, allow(missing_docs))]
        struct Label(String);

        assert_eq!(Label::new("x").into_inner(), "x");
    }

    #[test]
    fn attr_above_nutype_is_forwarded() {
        #[allow(missing_docs)]
        #[nutype(derive(Debug))]
        struct Above(i32);

        assert_eq!(Above::new(1).into_inner(), 1);
    }

    #[test]
    fn attrs_mix_with_doc_comments() {
        /// Mixed is documented.
        #[nutype(derive(Debug))]
        #[repr(transparent)]
        #[allow(missing_docs)]
        struct Mixed(f64);

        assert_eq!(Mixed::new(1.5).into_inner(), 1.5);
    }

    // rustc strips item-level `cfg` before the attribute macro expands, so
    // `#[cfg(...)]` composes natively with #[nutype], in both positions:
    // a false predicate removes the whole type (nutype never runs), a true
    // predicate just drops the cfg attribute.
    #[test]
    fn cfg_on_the_item_is_handled_natively_by_rustc() {
        #[cfg(test)]
        #[nutype(derive(Debug))]
        struct AboveTrue(i32);

        #[nutype(derive(Debug))]
        #[cfg(test)]
        struct BelowTrue(i32);

        // `any()` is always false: the whole item (including #[nutype]) is
        // stripped before expansion. If nutype were invoked on it, this would
        // not compile (the type is referenced nowhere, but a nutype parse
        // error would still abort compilation).
        #[nutype(derive(Debug), validate(unknown_validator_that_would_error))]
        #[cfg(any())]
        struct StrippedEntirely(i32);

        assert_eq!(AboveTrue::new(1).into_inner(), 1);
        assert_eq!(BelowTrue::new(2).into_inner(), 2);
    }

    #[test]
    fn const_fn_with_forwarded_attrs() {
        #[nutype(const_fn, derive(Debug), validate(greater_or_equal = 0))]
        #[repr(transparent)]
        struct Positive(i32);

        const P: Positive = match Positive::try_new(3) {
            Ok(value) => value,
            Err(_) => panic!("invalid"),
        };
        assert_eq!(P.into_inner(), 3);
    }
}

// ---------------------------------------------------------------------------
// Field-level attribute forwarding
// ---------------------------------------------------------------------------
mod field_attr_forwarding {
    use super::*;

    #[test]
    fn builtin_field_attr_on_string_kind() {
        #[nutype(derive(Debug))]
        struct Name(#[allow(unused)] String);

        assert_eq!(Name::new("x").into_inner(), "x");
    }

    #[test]
    fn builtin_field_attr_on_integer_kind() {
        #[nutype(derive(Debug))]
        struct Count(#[allow(unused)] u32);

        assert_eq!(Count::new(7).into_inner(), 7);
    }

    // Regression guard: for "any" inner types the field used to be embedded
    // into the generated code as a whole `syn::Field`, leaking field
    // attributes into type positions (e.g. `fn try_new(raw_value: #[attr] Vec<u8>)`).
    #[test]
    fn builtin_field_attr_on_any_kind() {
        #[nutype(derive(Debug))]
        struct Wrapper(#[allow(unused)] Vec<u8>);

        assert_eq!(Wrapper::new(vec![1]).into_inner(), vec![1]);
    }

    #[test]
    fn field_attr_on_generic_newtype() {
        #[nutype(derive(Debug))]
        struct Many<T>(#[allow(unused)] Vec<T>);

        assert_eq!(Many::new(vec![1, 2]).into_inner(), vec![1, 2]);
    }

    #[test]
    fn cfg_attr_wrapped_field_attr() {
        #[nutype(derive(Debug))]
        struct Guarded(#[cfg_attr(test, allow(unused))] i64);

        assert_eq!(Guarded::new(3).into_inner(), 3);
    }
}

// NOTE: the proof that forwarded field attributes are read by a real
// third-party derive (via derive_unchecked) lives in
// examples/derive_unchecked_example, because the test_suite deliberately does
// not enable the `derive_unchecked` feature (several UI fixtures pin the
// feature-off error messages and would flip under --all-features).

// ---------------------------------------------------------------------------
// Serde customization: with / serialize_with / deserialize_with / transparent
// ---------------------------------------------------------------------------
#[cfg(feature = "serde")]
mod serde_customization {
    use super::*;

    /// Toy codec: Vec<u8> as a hex string.
    pub mod hex_bytes {
        pub fn serialize<S: serde::Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
            let hex: String = v.iter().map(|b| format!("{b:02x}")).collect();
            s.serialize_str(&hex)
        }

        pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
            use serde::Deserialize;
            let s = String::deserialize(d)?;
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(serde::de::Error::custom))
                .collect()
        }
    }

    pub fn ser_i32_as_string<S: serde::Serializer>(v: &i32, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }

    pub fn de_i32_from_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
        use serde::Deserialize;
        let s = String::deserialize(d)?;
        s.parse::<i32>().map_err(serde::de::Error::custom)
    }

    pub fn ser_string_uppercase<S: serde::Serializer>(v: &String, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_uppercase())
    }

    mod with_module {
        use super::*;

        #[test]
        fn roundtrip_through_custom_codec() {
            #[nutype(
                validate(predicate = |v| !v.is_empty()),
                derive(Debug, PartialEq, Serialize, Deserialize),
            )]
            struct Token(#[serde(with = "hex_bytes")] Vec<u8>);

            let token = Token::try_new(vec![0xde, 0xad]).unwrap();
            let json = serde_json::to_string(&token).unwrap();
            assert_eq!(json, "\"dead\"");

            let back: Token = serde_json::from_str(&json).unwrap();
            assert_eq!(back, token);
        }

        #[test]
        fn validation_still_runs_after_custom_deserialization() {
            #[nutype(
                validate(predicate = |v| !v.is_empty()),
                derive(Debug, PartialEq, Serialize, Deserialize),
            )]
            struct Token(#[serde(with = "hex_bytes")] Vec<u8>);

            // Empty hex string decodes fine, but violates the predicate.
            let err = serde_json::from_str::<Token>("\"\"").unwrap_err();
            assert!(
                err.to_string().contains("Expected valid Token"),
                "unexpected error: {err}"
            );
        }

        #[test]
        fn sanitization_still_runs_after_custom_deserialization() {
            #[nutype(
                sanitize(with = |mut v| { v.sort(); v }),
                derive(Debug, PartialEq, Serialize, Deserialize),
            )]
            struct SortedBytes(#[serde(with = "hex_bytes")] Vec<u8>);

            // 0x02, 0x01 in the wire format must come out sorted.
            let value: SortedBytes = serde_json::from_str("\"0201\"").unwrap();
            assert_eq!(value.into_inner(), vec![0x01, 0x02]);
        }
    }

    mod serialize_with_only {
        use super::*;

        #[test]
        fn integer_kind() {
            #[nutype(derive(Debug, Serialize))]
            struct Id(#[serde(serialize_with = "ser_i32_as_string")] i32);

            let json = serde_json::to_string(&Id::new(7)).unwrap();
            assert_eq!(json, "\"7\"");
        }

        #[test]
        fn string_kind() {
            #[nutype(sanitize(trim), derive(Debug, Serialize))]
            struct Code(#[serde(serialize_with = "ser_string_uppercase")] String);

            let json = serde_json::to_string(&Code::new("  abc ")).unwrap();
            assert_eq!(json, "\"ABC\"");
        }
    }

    mod deserialize_with_only {
        use super::*;

        #[test]
        fn integer_kind_with_validation() {
            #[nutype(validate(greater_or_equal = 0), derive(Debug, Deserialize))]
            struct Id(#[serde(deserialize_with = "de_i32_from_string")] i32);

            let id: Id = serde_json::from_str("\"7\"").unwrap();
            assert_eq!(id.into_inner(), 7);

            // Custom deserialization succeeded, validation must still reject.
            let err = serde_json::from_str::<Id>("\"-3\"").unwrap_err();
            assert!(
                err.to_string().contains("Expected valid Id"),
                "unexpected error: {err}"
            );
        }
    }

    mod transparent {
        use super::*;

        #[test]
        fn serialization_equals_inner_value_json_and_ron() {
            #[nutype(derive(Debug, PartialEq, Serialize, Deserialize))]
            #[serde(transparent)]
            struct Meters(i32);

            let m = Meters::new(5);
            assert_eq!(
                serde_json::to_string(&m).unwrap(),
                serde_json::to_string(&5).unwrap()
            );
            // The principled framing assertion: transparent output is exactly
            // the inner value's own serialization, in any format.
            assert_eq!(ron::to_string(&m).unwrap(), ron::to_string(&5).unwrap());

            // And it deserializes back from the inner value's serialization.
            let from_ron: Meters = ron::from_str(&ron::to_string(&5).unwrap()).unwrap();
            assert_eq!(from_ron, m);
            let from_json: Meters = serde_json::from_str("5").unwrap();
            assert_eq!(from_json, m);
        }

        #[test]
        fn validation_still_runs() {
            #[nutype(validate(greater_or_equal = 0), derive(Debug, Serialize, Deserialize))]
            #[serde(transparent)]
            struct Positive(i32);

            let err = serde_json::from_str::<Positive>("-1").unwrap_err();
            assert!(
                err.to_string().contains("Expected valid Positive"),
                "unexpected error: {err}"
            );
        }

        #[test]
        fn float_kind() {
            #[nutype(validate(finite), derive(Debug, PartialEq, Serialize, Deserialize))]
            #[serde(transparent)]
            struct Weight(f64);

            let w = Weight::try_new(72.5).unwrap();
            let json = serde_json::to_string(&w).unwrap();
            assert_eq!(json, "72.5");
            let back: Weight = serde_json::from_str(&json).unwrap();
            assert_eq!(back, w);
        }

        #[test]
        fn string_kind_with_sanitization() {
            #[nutype(sanitize(trim), derive(Debug, PartialEq, Serialize, Deserialize))]
            #[serde(transparent)]
            struct Login(String);

            let login: Login = serde_json::from_str("\"  alice \"").unwrap();
            assert_eq!(login.into_inner(), "alice");
        }

        #[test]
        fn generic_newtype() {
            #[nutype(derive(Debug, PartialEq, Serialize, Deserialize))]
            #[serde(transparent)]
            struct AnyVal<T>(T);

            let v = AnyVal::new(33);
            let json = serde_json::to_string(&v).unwrap();
            assert_eq!(json, "33");
            let back: AnyVal<i32> = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }

        #[test]
        fn combined_with_custom_functions() {
            #[nutype(
                validate(predicate = |v| !v.is_empty()),
                derive(Debug, PartialEq, Serialize, Deserialize),
            )]
            #[serde(transparent)]
            struct Token(#[serde(with = "hex_bytes")] Vec<u8>);

            let token = Token::try_new(vec![0xbe, 0xef]).unwrap();
            let json = serde_json::to_string(&token).unwrap();
            assert_eq!(json, "\"beef\"");
            let back: Token = serde_json::from_str(&json).unwrap();
            assert_eq!(back, token);

            // Validation still gates the custom deserialization.
            let err = serde_json::from_str::<Token>("\"\"").unwrap_err();
            assert!(
                err.to_string().contains("Expected valid Token"),
                "unexpected error: {err}"
            );
        }
    }

    mod conditional_derives {
        use super::*;

        // The serde derives come from a cfg_attr group; the field serde attrs
        // must thread through the conditional generation path as well.
        #[test]
        fn cfg_attr_serde_derive_with_field_attrs() {
            #[nutype(
                derive(Debug, PartialEq),
                cfg_attr(feature = "serde", derive(Serialize, Deserialize))
            )]
            struct CondToken(#[serde(with = "hex_bytes")] Vec<u8>);

            let token = CondToken::new(vec![0x0a, 0xff]);
            let json = serde_json::to_string(&token).unwrap();
            assert_eq!(json, "\"0aff\"");
            let back: CondToken = serde_json::from_str(&json).unwrap();
            assert_eq!(back, token);
        }

        #[test]
        fn cfg_attr_serde_derive_with_transparent() {
            #[nutype(
                derive(Debug, PartialEq),
                cfg_attr(feature = "serde", derive(Serialize, Deserialize))
            )]
            #[serde(transparent)]
            struct CondMeters(i32);

            let m = CondMeters::new(8);
            assert_eq!(ron::to_string(&m).unwrap(), ron::to_string(&8).unwrap());
        }
    }

    // The default (non-transparent) newtype framing must stay intact for
    // types that use custom functions: RON roundtrip exercises the
    // visit_newtype_struct path.
    mod default_framing_preserved {
        use super::*;

        #[test]
        fn ron_roundtrip_with_custom_codec() {
            #[nutype(derive(Debug, PartialEq, Serialize, Deserialize))]
            struct Blob(#[serde(with = "hex_bytes")] Vec<u8>);

            let blob = Blob::new(vec![0x01, 0x02]);
            let ron_str = ron::to_string(&blob).unwrap();
            let back: Blob = ron::from_str(&ron_str).unwrap();
            assert_eq!(back, blob);
        }
    }
}

#[cfg(all(feature = "serde", feature = "rust_decimal"))]
mod serde_customization_decimal {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn transparent_decimal() {
        #[nutype(
            validate(greater_or_equal = 0),
            derive(Debug, PartialEq, Serialize, Deserialize)
        )]
        #[serde(transparent)]
        struct Price(Decimal);

        let price = Price::try_new(Decimal::new(1234, 2)).unwrap(); // 12.34
        let json = serde_json::to_string(&price).unwrap();
        assert_eq!(json, serde_json::to_string(&Decimal::new(1234, 2)).unwrap());
        let back: Price = serde_json::from_str(&json).unwrap();
        assert_eq!(back, price);
    }
}
