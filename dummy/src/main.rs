use nutype::nutype;

// Inner custom type, which is unknown to nutype
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn magnitude(&self) -> f64 {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        f64::sqrt(x * x + y * y)
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl std::str::FromStr for Point {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .split(',')
            .map(|part| part.parse::<i32>().map_err(|_| "Invalid Integer"))
            .collect::<Result<Vec<_>, _>>()?;

        if items.len() != 2 {
            return Err("Point must be two comma separated integers");
        }
        let x = items[0];
        let y = items[1];
        Ok(Point::new(x, y))
    }
}

#[nutype(derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, AsRef, Into, From, Deref, Borrow,
    FromStr
))]
pub struct Location(Point);

fn main() {}
