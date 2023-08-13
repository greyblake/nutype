mod encapsulated {
    use nutype::nutype;

    #[nutype(validate(greater_or_equal = 0.0, less_or_equal = 100.0))]
    struct Percentage(f32);
}

type TheError = encapsulated::PercentageError;

fn main () {}
