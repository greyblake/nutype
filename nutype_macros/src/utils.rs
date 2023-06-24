// Example:
//
//     let msg = match_feature!("foo",
//         on => "Foo is on!",
//         off => "Foo is off",
//     );
//
macro_rules! match_feature {
    // Canonical
    (
        $feature_name:literal,
        on => $on_code:expr,
        off => $off_code:expr,
    ) => {{
        #[cfg(feature = $feature_name)]
        {
            $on_code
        }

        #[cfg(not(feature = $feature_name))]
        {
            $off_code
        }
    }};

    // Alternative: without trailing comma
    (
        $feature_name:literal,
        on => $on_code:expr,
        off => $off_code:expr
    ) => {{
        match_feature!($feature_name,
            on => $on_code,
            off => $off_code,
        )
    }}
}

pub(crate) use match_feature;
