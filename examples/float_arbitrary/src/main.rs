use arbitrary::Arbitrary;
use nutype::nutype;

#[nutype(
    derive(Debug, Arbitrary),
    sanitize(with = |x| x),
)]
struct UnrestrictedFloatNumber(f64);

#[nutype(derive(Debug, Arbitrary), validate(finite))]
struct FiniteF64(f64);

#[nutype(derive(Debug, Arbitrary), validate(finite))]
struct FiniteF32(f32);

#[nutype(derive(Debug, Arbitrary), validate(greater_or_equal = -64.4))]
struct GreaterOrEqualF64(f64);

#[nutype(derive(Debug, Arbitrary), validate(greater_or_equal = 32.2))]
struct GreaterOrEqualF32(f32);

#[nutype(derive(Debug, Arbitrary), validate(greater = -64.0))]
struct GreaterF64(f64);

#[nutype(derive(Debug, Arbitrary), validate(greater = 32.0))]
struct GreaterF32(f32);

#[nutype(derive(Debug, Arbitrary), validate(greater = -1.0, less = 1.0))]
struct GreaterAndLessF64(f64);

#[nutype(derive(Debug, Arbitrary), validate(greater = -10.0, less = 10.0))]
struct GreaterAndLessF32(f32);

#[nutype(derive(Debug, Arbitrary), validate(greater_or_equal = -10.0, less = 10.0))]
struct GreaterOrEqualAndLessF32(f32);

#[nutype(derive(Debug, Arbitrary), validate(greater = -10.0, less_or_equal = 10.0))]
struct GreaterAndLessOrEqualF32(f32);

#[nutype(derive(Debug, Arbitrary), validate(greater = -1.0, less_or_equal = -0.5))]
struct GreaterOrEqualAndLessOrEqualF64(f64);

fn main() {
    arbtest::builder().run(|u| {
        let _num = UnrestrictedFloatNumber::arbitrary(u)?.into_inner();
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f64 = FiniteF64::arbitrary(u)?.into_inner();
        assert!(value.is_finite());
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f32 = FiniteF32::arbitrary(u)?.into_inner();
        assert!(value.is_finite());
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f64 = GreaterOrEqualF64::arbitrary(u)?.into_inner();
        assert!(value >= -64.4);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f32 = GreaterOrEqualF32::arbitrary(u)?.into_inner();
        assert!(value >= 32.2);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f64 = GreaterF64::arbitrary(u)?.into_inner();
        assert!(value > -64.0);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f32 = GreaterF32::arbitrary(u)?.into_inner();
        assert!(value > 32.0);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f64 = GreaterAndLessF64::arbitrary(u)?.into_inner();
        assert!(value > -1.0 && value < 1.0);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f32 = GreaterAndLessF32::arbitrary(u)?.into_inner();
        assert!(value > -10.0 && value < 10.0);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f32 = GreaterOrEqualAndLessF32::arbitrary(u)?.into_inner();
        assert!((-10.0..10.0).contains(&value));
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f32 = GreaterAndLessOrEqualF32::arbitrary(u)?.into_inner();
        assert!(value > -10.0 && value <= 10.0);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value: f64 = GreaterOrEqualAndLessOrEqualF64::arbitrary(u)?.into_inner();
        assert!((-1.0..=-0.5).contains(&value));
        Ok(())
    });
}
