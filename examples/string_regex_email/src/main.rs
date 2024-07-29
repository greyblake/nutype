use std::sync::LazyLock;

use nutype::nutype;
use regex::Regex;

// Note: this regex is very simplified.
// In reality you'd like to use a more sophisticated regex for email validation.
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^\\w+@\\w+\\.\\w+$").unwrap());

// Note: technically the local part of email address is case-sensitive, but in practice all the
// popular email services (e.g. gmail) make it case-insensitive, so applying `lowercase` is OK.
#[nutype(
    sanitize(trim, lowercase),
    validate(
        len_char_min = 5,
        len_char_max = 20,
        regex = EMAIL_REGEX,
    ),
    derive(Debug, PartialEq, AsRef),
)]
struct Email(String);

fn main() {
    // Too short
    assert_eq!(Email::try_new("a@b"), Err(EmailError::LenCharMinViolated));

    // Too long
    assert_eq!(
        Email::try_new("abcedfghijklmnopqrstuvwxyz@b.example"),
        Err(EmailError::LenCharMaxViolated)
    );

    // Does not match the regex
    assert_eq!(Email::try_new("foo@barcom"), Err(EmailError::RegexViolated));

    // A valid email
    let email: Email = Email::try_new("\t Nutype@Example.Com \n").unwrap();

    // The underlying string that represents the email address is sanitized
    assert_eq!(email.as_ref(), "nutype@example.com");
}
