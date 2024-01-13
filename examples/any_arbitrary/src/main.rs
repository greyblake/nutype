use arbitrary::Arbitrary;
use nutype::nutype;

#[derive(Arbitrary)]
struct Point {
    x: i32,
    y: i32,
}

// Inner type is custom type Point, which implements Arbitrary.
// There is no validation, but custom sanitization.
// Deriving Arbitrary with custom validation would not be possible in this case.
#[nutype(
    derive(Arbitrary),
    sanitize(with = |mut point| {
        point.x = point.x.clamp(0, 100);
        point.y = point.y.clamp(-200, 200);
        point
    })
)]
pub struct Location(Point);

fn main() {
    arbtest::builder().run(|u| {
        let location = u.arbitrary::<Location>()?;
        let point = location.into_inner();
        assert!(point.x >= 0 && point.x <= 100);
        assert!(point.y >= -200 && point.y <= 200);
        Ok(())
    });
}
