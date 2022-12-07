use std::fmt::Debug;

use proc_macro2::Span;
use syn::spanned::Spanned;

pub trait Kind {
    type Kind: PartialEq + Debug;

    fn kind(&self) -> Self::Kind;
}

#[derive(Debug)]
pub struct SpannedItem<T> {
    pub item: T,
    pub span: Span,
}

impl<T> Spanned for SpannedItem<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: Kind> Kind for SpannedItem<T> {
    type Kind = <T as Kind>::Kind;

    fn kind(&self) -> Self::Kind {
        self.item.kind()
    }
}
