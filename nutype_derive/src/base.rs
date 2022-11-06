use proc_macro2::Span;

pub trait Kind {
    type Kind;

    fn kind(&self) -> Self::Kind;
}

#[derive(Debug)]
pub struct Spanned<T> {
    pub item: T,
    pub span: Span,
}

// impl<T> Spanned<T> {
//     pub fn new(item: T, span: Span) -> Self {
//         Self { item, span }
//     }
// }

impl<T: Kind> Kind for Spanned<T> {
    type Kind = <T as Kind>::Kind;

    fn kind(&self) -> Self::Kind {
        self.item.kind()
    }
}
