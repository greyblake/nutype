use nutype::nutype;

#[nutype(validate(max = 12.34))]
#[derive(FromStr)]
pub struct Dist(f64);

fn main() {
    let dist: Dist = "11.4".parse().unwrap();
    println!("dist = {}", dist.into_inner());
}
