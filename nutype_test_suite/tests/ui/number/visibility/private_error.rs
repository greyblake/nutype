mod encapsulated {
    use nutype::nutype;

    #[nutype(validate(min = 0, max = 100))]
    struct Percentage(i32);
}

type TheError = encapsulated::PercentageError;

fn main () {}
