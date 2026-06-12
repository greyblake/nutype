// WATCH OUT: derive_unchecked() allows to derive any trait even those that may create loopholes
// in the validation and sanitization logic!
use derive_more::{Deref, DerefMut};
use nutype::nutype;

#[nutype(
    derive(Debug, AsRef),
    derive_unchecked(Deref, DerefMut),
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

#[cfg(test)]
mod tests {
    use super::*;

    // Proof that field attributes are forwarded to the generated field and
    // read by a real third-party derive: `arbitrary`'s derive honors
    // `#[arbitrary(value = ...)]`. If the attribute were dropped (the old
    // nutype behavior), `arbitrary` would produce 0 from empty input
    // instead of 42.
    #[test]
    fn field_attr_reaches_third_party_derive() {
        use arbitrary::{Arbitrary, Unstructured};

        #[nutype(derive(Debug), derive_unchecked(::arbitrary::Arbitrary))]
        struct Fixed(#[arbitrary(value = 42)] i32);

        let mut u = Unstructured::new(&[]);
        let value = Fixed::arbitrary(&mut u).unwrap();
        assert_eq!(value.into_inner(), 42);
    }

    // Type-level attributes are forwarded as well: `derive_more::Display`
    // reads the `#[display(...)]` attribute from the type.
    #[test]
    fn type_attr_reaches_third_party_derive() {
        #[nutype(derive(Debug), derive_unchecked(derive_more::Display))]
        #[display("ID-{_0}")]
        struct Id(u32);

        assert_eq!(Id::new(7).to_string(), "ID-7");
    }
}
