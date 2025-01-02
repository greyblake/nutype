use nutype::nutype;

#[nutype(
    const_fn,
    derive(Debug),
    validate(greater_or_equal = -273.15),
)]
pub struct Celsius(f64);

impl Celsius {
    pub const fn new_const(value: f64) -> Self {
        match Self::try_new(value) {
            Ok(value) => value,
            Err(_e) => {
                panic!("Failed to create Celsius");
            }
        }
    }
}

const WATER_BOILING_POINT: Celsius = Celsius::new_const(100.0);

fn main() {
    println!("{:?}", WATER_BOILING_POINT);
}
