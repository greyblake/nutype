use nutype::nutype;

// schemars attributes are not supported (yet): nutype generates its own
// JsonSchema impl, so the helper attr would not resolve.
#[nutype(derive(Debug))]
#[schemars(title = "Name")]
pub struct Name(String);

fn main() {}
