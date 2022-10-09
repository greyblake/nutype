/*
*/
// use std::convert::TryFrom;
use nutype_derive::nutype;

use core::convert::TryFrom;


// New crate name ideas:
// * newty (unfortunately taken :( )
// * typex
// * tipo
// * powertype
// * nutype
// * xtype


// struct Foo;
//
// impl TryFrom<String> for Foo {
//     type Error = ();
//
//     fn try_from(s: String) -> Result<Foo, ()> {
//         Ok(Foo)
//     }
// }



// #[derive(Debug)]
#[nutype(
    derive(Debug, PartialEq, AsRef)
    sanitize(trim, lowercase)
    validate(
        error_type(EmailError),
        min_len(6),
        max_len(255)
    )
)]
pub struct Email(String);


// #[nutype(
//     sanitize(trim, lowercase),
//     validate(min_len = 10))
//     derive(Debug, Clone)
// ]
// pub struct UserName(String);


/*
    sanitize = [trim, lowercase],
    validate = [min_len = 6, max_len = 255]
    validate(min_len = 6, max_len = 255)
*/

/*

mod newty_email {
    #[derive(Debug, PartialEq)]
    pub struct Email(String);

    // Syntax sugar
    //
    // impl TryFrom<&str> for Email {
    //     type Error = EmailError;

    //     fn try_from(raw: &str) -> Result<Email, Self::Error> {
    //         Email::try_from(raw.to_string())
    //     }
    // }

    impl TryFrom<String> for Email {
        type Error = EmailError;

        fn try_from(raw: String) -> Result<Email, Self::Error> {
            let sanitized_value = sanitize(raw);
            let validated_value = validate(sanitized_value)?;
            Ok(Email(validated_value))
        }
    }

    impl AsRef<str> for Email {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }


    fn sanitize(raw: String) -> String {
        raw.trim().to_lowercase()
    }

    #[derive(Debug, PartialEq)]
    pub enum EmailError {
        TooShort,
        TooLong,
    }

    fn validate(value: String) -> Result<String, EmailError> {
        if value.len() < 6 {
            return Err(EmailError::TooShort);
        }
        if value.len() > 255 {
            return Err(EmailError::TooLong);
        }
        return Ok(value);
    }
}

use newty_email::Email;
use newty_email::EmailError;


// fn change_email(mut email: Email) -> Email {
//     email.0 = "Bang".to_string();
//     email
// }





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_trim_email_and_lowercase() {
        let email: Email = Email::try_from(" ME@example.com \n".to_string()).unwrap();
        assert_eq!(email.as_ref(), "me@example.com");
    }

    #[test]
    fn should_email_too_short() {
        let input = "foo".to_string();
        assert_eq!(
            Email::try_from(input),
            Err(EmailError::TooShort));
    }
}

*/

fn main() {
    //let raw_email = "\tBlake13@gmail.COM  \n";
    let raw_email = "\tBlake13@gmail.COM  \n";
    let email = Email::try_from(raw_email.to_string()).unwrap();
    println!("\n\nemail = {:?}\n\n", email);
}
