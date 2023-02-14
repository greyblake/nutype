mod encapsulated {
    use nutype::nutype;

    #[nutype(validate(not_empty))]
    struct Name(String);
}

type TheError = encapsulated::NameError;

fn main() {}
