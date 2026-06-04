#[cfg(feature = "ui")]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}

// Decimal UI fixtures are split by whether the `rust_decimal` feature is enabled,
// because the expected error differs:
// * with the feature ON, fixtures exercise Decimal-specific validation errors;
// * with the feature OFF, the fixture exercises the "enable the feature" error.
// They live outside `tests/ui/**` so the main glob above never picks them up,
// which keeps `--all-features` (ui + rust_decimal) consistent.
#[cfg(all(feature = "ui", feature = "rust_decimal"))]
#[test]
fn ui_decimal_on() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_decimal_on/**/*.rs");
}

#[cfg(all(feature = "ui", not(feature = "rust_decimal")))]
#[test]
fn ui_decimal_off() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_decimal_off/**/*.rs");
}

// Fixtures that additionally need the `arbitrary` feature to reach the
// Decimal-specific rejection (otherwise `derive(Arbitrary)` is rejected earlier
// by the feature gate, which would make the snapshot feature-sensitive).
#[cfg(all(feature = "ui", feature = "rust_decimal", feature = "arbitrary"))]
#[test]
fn ui_decimal_on_arbitrary() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_decimal_on_arbitrary/**/*.rs");
}
