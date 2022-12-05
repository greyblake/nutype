pub mod error;
pub mod parse_error;
pub mod traits;

use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Inject an inner type into a closure, so compiler does not complain if the token stream matchers
/// the expected closure pattern.
///
/// Input:
///   |s| s.trim().to_lowercase()
/// Output:
///   |s: String| s.trim().to_lowercase()
pub fn type_custom_sanitizier_closure(
    closure_or_func_path: &TokenStream,
    typ: TokenStream,
) -> TokenStream {
    let mut ts: Vec<TokenTree> = closure_or_func_path.clone().into_iter().collect();

    // Check if the tokens match `|s|` pattern
    // If so, inject the type, e.g. `|s: String|`
    if ts.len() >= 3 && is_pipe(&ts[0]) && is_ident(&ts[1]) && is_pipe(&ts[2]) {
        let colon = TokenTree::Punct(Punct::new(':', Spacing::Alone));
        ts.insert(2, colon);
        for (index, tok) in typ.into_iter().enumerate() {
            let pos = index + 3;
            ts.insert(pos, tok);
        }
    }

    ts.into_iter().collect()
}

fn is_pipe(token: &TokenTree) -> bool {
    match token {
        TokenTree::Punct(ref punct) => punct.as_char() == '|',
        _ => false,
    }
}

fn is_ident(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Ident(_))
}

pub fn gen_module_name_for_type(type_name: &Ident) -> Ident {
    let module_name = format!("__nutype_private_{type_name}__");
    Ident::new(&module_name, Span::call_site())
}
