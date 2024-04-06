use arbitrary::Arbitrary;
use arbtest::arbtest;
use nutype::nutype;

fn main() {
    should_generate_arbitrary_string_without_validation_with_respect_to_sanitizers();
    should_generate_arbitrary_string_with_trim_and_min_len_validation();
    should_respect_not_empty_validation_with_trim();
    should_respect_not_empty_validation_without_trim();
    should_respect_len_char_max();
    should_respec_both_len_boundaries();
}

fn should_generate_arbitrary_string_without_validation_with_respect_to_sanitizers() {
    #[nutype(sanitize(lowercase), derive(Arbitrary, Debug))]
    struct LowercaseString(String);

    arbtest(|u| {
        let s = LowercaseString::arbitrary(u)?.into_inner();
        assert_eq!(s.to_lowercase(), s);
        Ok(())
    });
}

fn should_generate_arbitrary_string_with_trim_and_min_len_validation() {
    #[nutype(sanitize(trim), validate(len_char_min = 3), derive(Arbitrary, Debug))]
    struct Name(String);

    arbtest(|u| {
        let s = Name::arbitrary(u)?.into_inner();
        assert_eq!(s.trim(), s);
        assert!(s.chars().count() >= 3);
        Ok(())
    });
}

fn should_respect_not_empty_validation_with_trim() {
    #[nutype(sanitize(trim), validate(not_empty), derive(Arbitrary, Debug))]
    struct Title(String);

    arbtest(|u| {
        let s = Title::arbitrary(u)?.into_inner();
        assert_eq!(s.trim(), s);
        assert!(!s.is_empty());
        Ok(())
    });
}

fn should_respect_not_empty_validation_without_trim() {
    #[nutype(validate(not_empty), derive(Arbitrary, Debug))]
    struct Description(String);

    arbtest(|u| {
        let s = Description::arbitrary(u)?.into_inner();
        assert!(!s.is_empty());
        Ok(())
    });
}

fn should_respect_len_char_max() {
    #[nutype(validate(len_char_max = 7), derive(Arbitrary, Debug))]
    struct Text(String);

    arbtest(|u| {
        let s = Text::arbitrary(u)?.into_inner();
        assert!(s.chars().count() <= 7);
        Ok(())
    });
}

fn should_respec_both_len_boundaries() {
    #[nutype(validate(len_char_min = 3, len_char_max = 5), derive(Arbitrary, Debug))]
    struct Text(String);

    arbtest(|u| {
        let s = Text::arbitrary(u)?.into_inner();
        assert!(s.chars().count() >= 3);
        assert!(s.chars().count() <= 5);
        Ok(())
    });
}
