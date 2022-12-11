use nutype::nutype;

#[nutype(validate(max = 12.34))]
#[derive(FromStr, Display)]
pub struct Dist(f64);

fn main() {
    let dist: Dist = "11.4".parse().unwrap();
    println!("dist = {}", dist.into_inner());

    let dresult = "12.0".parse::<Dist>();

    match dresult {
        Ok(d) => println!("dist = {d}"),
        Err(DistParseError::Validate(ve)) => {
            println!("ve = {ve:?}");
        }
        Err(DistParseError::Parse(pe)) => {
            println!("pe = {pe:?}");
        }
    }
}
