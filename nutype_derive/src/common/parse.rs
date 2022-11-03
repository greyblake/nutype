use std::{any::type_name, fmt::Debug, str::FromStr};

use proc_macro2::{Group, Ident, TokenTree};

/// ## Example
/// Input (token stream):
///     = 123
/// Output (parsed value):
///    123
pub fn parse_value_as<T, ITER>(mut iter: ITER) -> Result<(T, ITER), Vec<syn::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
    ITER: Iterator<Item = TokenTree>,
{
    let token_eq = iter.next().expect("Expected token `=`");
    assert_eq!(token_eq.to_string(), "=", "Expected token `=`");

    let token_value = iter.next().expect("Expected number");
    let str_value = token_value.to_string();
    let value: T = str_value.parse::<T>().map_err(|_err| {
        let msg = format!("Expected {}, got `{}`", type_name::<T>(), str_value);
        vec![syn::Error::new(token_value.span(), msg)]
    })?;
    Ok((value, iter))
}

pub fn try_unwrap_ident(token: TokenTree) -> Result<Ident, Vec<syn::Error>> {
    match token {
        TokenTree::Ident(ident) => Ok(ident),
        _ => {
            let error = syn::Error::new(token.span(), "#[nutype] expected ident");
            Err(vec![error])
        }
    }
}

pub fn try_unwrap_group(token: TokenTree) -> Result<Group, Vec<syn::Error>> {
    match token {
        TokenTree::Group(group) => Ok(group),
        _ => {
            let error = syn::Error::new(token.span(), "#[nutype] expected ident");
            Err(vec![error])
        }
    }
}
