mod encapsulated {
    use nutype::nutype;

    #[nutype(validate(present))]
    struct Name(String);
}

type TheError = encapsulated::NameError;

fn main () {}
