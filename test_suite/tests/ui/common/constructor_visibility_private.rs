mod inner {
    use nutype::nutype;

    // Type with private constructor - only accessible within this module
    #[nutype(
        sanitize(trim),
        constructor(visibility = private),
        derive(Debug, AsRef),
    )]
    pub struct PrivateName(String);
}

fn main() {
    // This should fail because new() is private to the `inner` module
    let _name = inner::PrivateName::new("test");
}
