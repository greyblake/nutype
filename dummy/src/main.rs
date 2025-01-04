use nutype::nutype;

#[nutype(
    const_fn,
    derive(Debug),
    validate(greater_or_equal = -273.15),
)]
pub struct Celsius(f64);

macro_rules! nutype_const {
    ($name:ident, $ty:ty, $value:expr) => {
        const $name: $ty = match <$ty>::try_new($value) {
            Ok(value) => value,
            Err(_) => panic!("Invalid value"),
        };
    };
}

nutype_const!(WATER_BOILING_POINT, Celsius, 100.0);

fn main() {
    println!("{:?}", WATER_BOILING_POINT);
}
