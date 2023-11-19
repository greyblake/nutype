use nutype::nutype;

#[nutype(
    validate(
        greater_or_equal = -100,
        less_or_equal = 100,
    ),
    derive(Debug, PartialEq)
)]
struct Volume(i8);

fn main() {
    // Too small
    assert_eq!(Volume::new(-101), Err(VolumeError::GreaterOrEqualViolated),);

    // Too big
    assert_eq!(Volume::new(101), Err(VolumeError::LessOrEqualViolated),);

    // Valid values
    assert_eq!(Volume::new(-100).unwrap().into_inner(), -100);
    assert_eq!(Volume::new(0).unwrap().into_inner(), 0);
    assert_eq!(Volume::new(100).unwrap().into_inner(), 100);
}
