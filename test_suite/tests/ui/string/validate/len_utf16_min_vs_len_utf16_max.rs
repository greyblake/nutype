use nutype::nutype;

#[nutype(validate(len_utf16_min = 127, len_utf16_max = 63))]
pub struct JsText(String);

fn main () {}
