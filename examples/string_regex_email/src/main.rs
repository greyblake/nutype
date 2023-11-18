use nutype::nutype;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Note: this regex is very simplified.
    // In reality you'd like to use a more sophisticated regex for email validation.
    static ref EMAIL_REGEX: Regex = Regex::new("^\\w+@\\w+\\.\\w+$").unwrap();
}

// Note: technically the local part of email address is case-sensitive, but in practice all the
// popular email services (e.g. gmail) make it case-insensitive, so applying `lowercase` is OK.
#[nutype(
    sanitize(trim, lowercase),
    validate(
        char_len_min = 5,
        char_len_max = 20,
        regex = EMAIL_REGEX,
    ),
    derive(Debug, PartialEq, AsRef),
)]
struct Email(String);

fn main() {
    // Too short
    assert_eq!(
        Email::new("a@b"),
        Err(EmailError::CharLenMinViolated)
    );

    // Too long
    assert_eq!(
        Email::new("abcedfghijklmnopqrstuvwxyz@b.example"),
        Err(EmailError::CharLenMaxViolated)
    );

    // Does not match the regex
    assert_eq!(
        Email::new("foo@barcom"),
        Err(EmailError::RegexViolated)
    );

    // A valid email
    let email: Email = Email::new("\t Nutype@Example.Com \n").unwrap();

    // The underlying string that represents the email address is sanitized
    assert_eq!(email.as_ref(), "nutype@example.com");
}
