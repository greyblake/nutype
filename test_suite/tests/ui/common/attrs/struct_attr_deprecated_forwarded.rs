#![deny(deprecated)]
use nutype::nutype;

// Observable proof that struct attributes are forwarded: if #[deprecated]
// reaches the generated struct, using the type triggers the deny(deprecated)
// error below. If the attribute were dropped, this file would compile.
#[nutype(derive(Debug))]
#[deprecated(note = "use NewName instead")]
pub struct OldName(String);

fn main() {
    let _ = OldName::try_new("hello");
}
