use nutype::nutype;

#[nutype(
    const_fn,
    validate(greater_or_equal = -273.15)
)]
pub struct Celcius(f64);

// This is expected to panic at compile time
const TOO_COLD: Celcius = match Celcius::try_new(-300.0) {
    Ok(celcius) => celcius,
    Err(_) => panic!("Invalid Celcius value"),
};

fn main() {}
