use syn::spanned::Spanned;

use crate::base::Kind;

pub fn validate_duplicates<T>(
    items: &[T],
    build_error_msg: impl Fn(<T as Kind>::Kind) -> String,
) -> Result<(), syn::Error>
where
    T: Spanned + Kind,
{
    if let Some((item1, item2)) = detect_items_of_same_kind(items) {
        assert_eq!(item1.kind(), item2.kind());
        let kind = item1.kind();
        let msg = build_error_msg(kind);
        let span = item1.span().join(item2.span()).expect("Items (validators or sanitizers) for the same type expected to be defined in the same file");
        let err = syn::Error::new(span, &msg);
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
