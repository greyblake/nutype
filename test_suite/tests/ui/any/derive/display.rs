use nutype::nutype;

struct Weight(f64);

#[nutype(derive(Display))]
struct IceCreamWeight(Weight);

fn main() {}
