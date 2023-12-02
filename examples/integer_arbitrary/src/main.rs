use arbitrary::Arbitrary;
use nutype::nutype;

// Inclusive boundaries. 1 and 6 are included, so the value can only be 1, 2, 3, 4, 5 or 6.
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 6),
    derive(Arbitrary, Debug)
)]
struct GermanTaxClass(i128);

// Exclusive boundaries.
//
// -2  and 2 are excluded, so the value can only be -1, 0 or 1.
#[nutype(
    validate(greater = -2, less = 2),
    derive(Arbitrary, Debug),
)]
struct MinusOneOrZeroOrOne(i128);

// Since the upper limit for i8 is 127, the GreaterThan125 can only be 126 or 127.
#[nutype(validate(greater = 125), derive(Arbitrary, Debug))]
struct GreaterThan125(i8);

// Since the upper limit for i8 is 127, the GreaterOrEqual125 can only be 125, 126 or 127.
#[nutype(validate(greater_or_equal = 125), derive(Arbitrary, Debug))]
struct GreaterOrEqual125(i8);

// u128::MIN is 0, so the LessThan2 can only be 0, 1
#[nutype(validate(less = 2), derive(Arbitrary, Debug))]
struct LessThan2(u128);

// u128::MIN is 0, so the LessOrEqual2 can only be 0, 1, 2
#[nutype(validate(less = 2), derive(Arbitrary, Debug))]
struct LessOrEqual2(u128);

fn main() {
    arbtest::builder().run(|u| {
        let tax_class = GermanTaxClass::arbitrary(u)?.into_inner();
        assert!(tax_class >= 1);
        assert!(tax_class <= 6);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value = GreaterThan125::arbitrary(u)?.into_inner();
        assert!(value == 126 || value == 127);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value = GreaterOrEqual125::arbitrary(u)?.into_inner();
        assert!(value == 125 || value == 126 || value == 127);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value = MinusOneOrZeroOrOne::arbitrary(u)?.into_inner();
        assert!(value == -1 || value == 0 || value == 1);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value = LessThan2::arbitrary(u)?.into_inner();
        assert!(value == 0 || value == 1);
        Ok(())
    });

    arbtest::builder().run(|u| {
        let value = LessOrEqual2::arbitrary(u)?.into_inner();
        assert!(value == 0 || value == 1 || value == 2);
        Ok(())
    });
}
