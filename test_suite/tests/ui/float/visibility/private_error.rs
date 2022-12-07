mod encapsulated {
    use nutype::nutype;

    #[nutype(validate(min = 0.0, max = 100.0))]
    struct Percentage(f32);
}

type TheError = encapsulated::PercentageError;

fn main () {}
