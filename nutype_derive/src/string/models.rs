use crate::models::{NewtypeMeta, RawNewtypeMeta};

#[derive(Debug, PartialEq)]
pub enum StringSanitizer {
    Trim,
    Lowecase,
    Uppercase,
}

#[derive(Debug, PartialEq)]
pub enum StringValidator {
    MinLen(usize),
    MaxLen(usize),
    Present,
}

pub type RawNewtypeStringMeta = RawNewtypeMeta<StringSanitizer, StringValidator>;
pub type NewtypeStringMeta = NewtypeMeta<StringSanitizer, StringValidator>;
