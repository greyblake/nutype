use nutype::nutype;

#[nutype(
    const_fn,
    validate(greater_or_equal = -273.15)
)]
pub struct Celsius(f64);

// This is expected to panic at compile time
const TOO_COLD: Celsius = match Celsius::try_new(-300.0) {
    Ok(celsius) => celsius,
    Err(_) => panic!("Invalid Celsius value"),
};

fn main() {}
