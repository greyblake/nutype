use kinded::Kinded;
use proc_macro2::Span;

use super::models::SpannedItem;

pub fn validate_duplicates<T>(
    items: &[SpannedItem<T>],
    build_error_msg: impl Fn(<T as Kinded>::Kind) -> String,
) -> Result<(), syn::Error>
where
    T: Kinded,
{
    if let Some((item1, item2)) = detect_items_of_same_kind(items) {
        assert_eq!(item1.kind(), item2.kind());
        let kind = item1.kind();
        let msg = build_error_msg(kind);
        let span = join_spans_or_last(item1.span(), item2.span());
        let err = syn::Error::new(span, msg);
        return Err(err);
    }
    Ok(())
}

fn detect_items_of_same_kind<T: Kinded>(items: &[T]) -> Option<(&T, &T)> {
    // Note: this has O(n^2) complexity, but it's not a problem, because size of collection is < 10.
    for (i1, item1) in items.iter().enumerate() {
        for (i2, item2) in items.iter().enumerate() {
            if i1 != i2 && item1.kind() == item2.kind() {
                return Some((item1, item2));
            }
        }
    }
    None
}

fn join_spans_or_last(span1: Span, span2: Span) -> Span {
    span1.join(span2).unwrap_or(span2)
}
