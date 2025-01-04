// Adding `const_fn` flag to #[nutype] macro will make the generated `new` and `try_new` functions
// be marked with `const` keyword.
use nutype::nutype;

#[nutype(const_fn, validate(greater_or_equal = 18))]
pub struct Age(u8);

// Unfortunately `Result::unwrap` is not a `const` function, so we have
// to unwrap it ourselves.
const DRINKING_AGE: Age = match Age::try_new(21) {
    Ok(age) => age,
    Err(_) => panic!("Invalid Age"),
};

// Until `const` functions are stabilized, we recommend to  define in application
// and reuse `nutype_const!` macro to create constants from `nutype` types
#[nutype(
    const_fn,
    validate(greater_or_equal = -1.0, less_or_equal = 1.0)
)]
struct Correlation(f32);

macro_rules! nutype_const {
    ($name:ident, $ty:ty, $value:expr) => {
        const $name: $ty = match <$ty>::try_new($value) {
            Ok(value) => value,
            Err(_) => panic!("Invalid value"),
        };
    };
}

nutype_const!(MAX_CORRELATION, Correlation, 1.0);

// Not recommended, but it's possible to use `new_unchecked` with `const_fn` flag.
#[nutype(
    const_fn,
    new_unchecked,
    validate(greater_or_equal = 1, less_or_equal = 6,)
)]
struct TaxClass(u8);

const DEFAULT_TAX_CLASS: TaxClass = unsafe { TaxClass::new_unchecked(1) };

// Note: `into_inner()` is const function as well.
nutype_const!(DOUBLE_AGE, Age, DRINKING_AGE.into_inner() * 2);

fn main() {
    assert_eq!(DRINKING_AGE.into_inner(), 21);
    assert_eq!(DEFAULT_TAX_CLASS.into_inner(), 1);
    assert_eq!(MAX_CORRELATION.into_inner(), 1.0);
    assert_eq!(DOUBLE_AGE.into_inner(), 42);
}
