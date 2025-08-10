// WATCH OUT: derive_unsafe() allows to derive any trait even those that may create loopholes
// in the validation and sanitization logic!
use derive_more::{Deref, DerefMut};
use nutype::nutype;

#[nutype(
    derive(Debug, AsRef),
    derive_unsafe(Deref, DerefMut),
    validate(greater_or_equal = 0.0, less_or_equal = 2.0)
)]
struct LlmTemperature(f64);

fn main() {
    let mut temperature = LlmTemperature::try_new(1.5).unwrap();

    // This is not what nutype is designed for!
    *temperature = 2.5;

    // OH no, we've just violated the validation rule!
    assert_eq!(temperature.as_ref(), &2.5);
}
