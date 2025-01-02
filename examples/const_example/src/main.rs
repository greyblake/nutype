// Adding `const_fn` flag to #[nutype] macro will make the generated `new` and `try_new` functions
// be marked with `const` keyword.
use nutype::nutype;

#[nutype(const_fn, validate(greater_or_equal = 18))]
pub struct Age(u8);

// Unfortunately `Result::unwrap` is not a `const` function, so we have
// to unwrap it ourselves.
const PENSION_AGE: Age = match Age::try_new(67) {
    Ok(age) => age,
    Err(_) => panic!("Invalid Age"),
};

// Not recommended, but it's possible to use `new_unchecked` with `const_fn` flag.
#[nutype(
    const_fn,
    new_unchecked,
    validate(greater_or_equal = 1, less_or_equal = 6,)
)]
struct TaxClass(u8);

const DEFAULT_TAX_CLASS: TaxClass = unsafe { TaxClass::new_unchecked(1) };

fn main() {
    assert_eq!(PENSION_AGE.into_inner(), 67);
    assert_eq!(DEFAULT_TAX_CLASS.into_inner(), 1);
}
