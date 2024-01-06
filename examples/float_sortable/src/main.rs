// This example shows how an f64 newtype can derive `Ord` and be sortable
// if it defines `finite` validation.

use nutype::nutype;

#[nutype(derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord), validate(finite))]
pub struct Width(f64);

// Corrrect (test typos)
fn main() {
    let raw_widths = vec![1.5, 1.4, 2.1, 1.8];

    // NOTE: sorting raw_widths is not possible, because f64 does not implement Ord.
    // raw_widths.sort();

    // So instead we can wrap f64 into Width, which implements Ord.
    // Ord is possible to safely derived, because there is `finite` validation in place,
    // which excluded NaN values.
    let mut widths: Vec<Width> = raw_widths
        .into_iter()
        .map(|w| Width::new(w).unwrap())
        .collect();

    // Now we can sort
    widths.sort();

    // Verify
    assert_eq!(
        widths,
        vec![
            Width::new(1.4).unwrap(),
            Width::new(1.5).unwrap(),
            Width::new(1.8).unwrap(),
            Width::new(2.1).unwrap(),
        ],
    )
}
