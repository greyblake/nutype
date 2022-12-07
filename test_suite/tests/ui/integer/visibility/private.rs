mod encapsulated {
    use nutype::nutype;

    #[nutype(sanitize(with = |n: i32| n.clamp(0, 100)))]
    struct Percentage(i32);

    fn inner_func() {
        Percentage::new(-13);
    }
}

fn main () {
    encapsulated::Percentage::new(144);
}
