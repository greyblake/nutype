mod encapsulated {
    use nutype::nutype;

    #[nutype(sanitize(trim))]
    struct Email(String);

    fn inner_func() {
        Email::new("foo@bar.com");
    }
}

fn main () {
    encapsulated::Email::new("foo@bar.com");
}
