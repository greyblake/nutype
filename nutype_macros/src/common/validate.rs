use crate::common::models::Kind;
use darling::util::SpannedValue;
use proc_macro2::Span;

pub fn validate_duplicates<T: Kind>(
    items: &[SpannedValue<T>],
    build_error_msg: impl Fn(<T as Kind>::Kind) -> String,
) -> Result<(), syn::Error> {
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

fn detect_items_of_same_kind<T: Kind>(items: &[T]) -> Option<(&T, &T)> {
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
