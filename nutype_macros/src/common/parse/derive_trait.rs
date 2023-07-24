use cfg_if::cfg_if;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};

use crate::common::models::{DeriveTrait, SpannedDeriveTrait};

impl Parse for SpannedDeriveTrait {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        let derive_trait = match ident.to_string().as_ref() {
            "Debug" => DeriveTrait::Debug,
            "Display" => DeriveTrait::Display,
            "Clone" => DeriveTrait::Clone,
            "Copy" => DeriveTrait::Copy,
            "PartialEq" => DeriveTrait::PartialEq,
            "Eq" => DeriveTrait::Eq,
            "PartialOrd" => DeriveTrait::PartialOrd,
            "Ord" => DeriveTrait::Ord,
            "FromStr" => DeriveTrait::FromStr,
            "AsRef" => DeriveTrait::AsRef,
            "Deref" => DeriveTrait::Deref,
            "TryFrom" => DeriveTrait::TryFrom,
            "From" => DeriveTrait::From,
            "Into" => DeriveTrait::Into,
            "Hash" => DeriveTrait::Hash,
            "Borrow" => DeriveTrait::Borrow,
            "Default" => DeriveTrait::Default,
            "Serialize" => {
                cfg_if! {
                    if #[cfg(feature = "serde")] {
                        DeriveTrait::SerdeSerialize
                    } else {
                        return Err(syn::Error::new(ident.span(), "To derive Serialize, the feature `serde` of the crate `nutype` needs to be enabled."));
                    }
                }
            }
            "Deserialize" => {
                cfg_if! {
                    if #[cfg(feature = "serde")] {
                        DeriveTrait::SerdeDeserialize
                    } else {
                        return Err(syn::Error::new(ident.span(), "To derive Deserialize, the feature `serde` of the crate `nutype` needs to be enabled."));
                    }
                }
            }
            "JsonSchema" => {
                cfg_if! {
                    if #[cfg(feature = "schemars08")] {
                        DeriveTrait::SchemarsJsonSchema
                    } else {
                        return Err(syn::Error::new(ident.span(), "To derive JsonSchema, the feature `schemars08` of the crate `nutype` needs to be enabled."));
                    }
                }
            }
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("Nutype cannot derive `{ident} is trait."),
                ));
            }
        };
        let spanned_trait = SpannedDeriveTrait {
            item: derive_trait,
            span: ident.span(),
        };
        Ok(spanned_trait)
    }
}
