mod encapsulated {
    use nutype::nutype;

    #[nutype(sanitize(with = |n| n.clamp(0.0, 100.0)))]
    struct Percentage(f32);

    fn inner_func() {
        Percentage::new(-13.1);
    }
}

fn main () {
    encapsulated::Percentage::new(144.2);
}
