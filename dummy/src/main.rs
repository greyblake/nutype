use nutype::nutype;

#[derive(Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

#[nutype(
    derive(Debug),
    sanitize(with = |p| {
        Point {
            x: p.x.clamp(0, 100),
            y: p.y.clamp(0, 100),
        }
    }),
    validate(predicate = |p| p.x > p.y),
)]
pub struct Pos(Point);

fn main() {}
