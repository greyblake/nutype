use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, DeriveInput, Visibility, spanned::Spanned};

use crate::{
    any::models::AnyInnerType,
    common::{
        models::{InnerType, Meta, SerdeCustomization, SpannedItem, TypeName},
        parse::{intercept_derive_macro, is_derive_attribute, is_doc_attribute},
    },
    float::models::FloatInnerType,
    integer::models::IntegerInnerType,
    string::models::StringInnerType,
};

pub fn parse_meta(token_stream: TokenStream) -> Result<Meta, syn::Error> {
    let input: DeriveInput = syn::parse(token_stream.into())?;

    let input_span = input.span();
    let DeriveInput {
        attrs,
        data,
        vis,
        ident: type_name,
        generics,
    } = input;

    let type_name = TypeName::new(type_name);

    intercept_derive_macro(&attrs)?;

    let mut serde_customization = SerdeCustomization::default();
    let PartitionedStructAttrs {
        doc_attrs,
        forwarded_attrs,
    } = partition_struct_attrs(attrs, &mut serde_customization)?;

    let data_struct = match &data {
        syn::Data::Struct(v) => v.clone(),
        _ => {
            let error =
                syn::Error::new(input_span, "#[nutype] can be used only with tuple structs.");
            return Err(error);
        }
    };

    let fields_unnamed = match data_struct.fields {
        syn::Fields::Unnamed(fu) => fu,
        _ => {
            let error =
                syn::Error::new(input_span, "#[nutype] can be used only with tuple structs.");
            return Err(error);
        }
    };

    let seg = fields_unnamed.unnamed.iter().next().ok_or_else(|| {
        let suggested_struct = quote::quote!(
            #vis struct #type_name(i32)
        )
        .to_string();
        let msg = format!(
            "Your wish to use #[nutype] with an empty tuple struct is respected.\n\
             But how about NO?\n\
             I bet you'll be luckier trying out something like this:\n\n\
             {suggested_struct};\n\n"
        );
        syn::Error::new(fields_unnamed.span(), msg)
    })?;
    validate_inner_field_visibility(&seg.vis)?;

    // Detach the field attributes:
    // * `#[serde(...)]` is consumed into `serde_customization`;
    // * the rest are forwarded verbatim onto the generated field.
    // The field embedded into `AnyInnerType` must NOT carry attributes,
    // because its tokens are also used in type positions (e.g. function
    // signatures), where attributes are not allowed.
    let mut seg = seg.clone();
    let field_attrs = core::mem::take(&mut seg.attrs);
    let inner_field_attrs = partition_field_attrs(field_attrs, &mut serde_customization)?;

    let type_path_str = seg.ty.clone().into_token_stream().to_string();

    // `into_token_stream().to_string()` renders paths with spaces around `::`
    // (e.g. `"rust_decimal :: Decimal"`), and that spacing is brittle across
    // syn versions. Strip all whitespace before matching the compact forms.
    let compact_type_path: String = type_path_str
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let inner_type = match compact_type_path.as_str() {
        "String" => InnerType::String(StringInnerType),
        "u8" => InnerType::Integer(IntegerInnerType::U8),
        "u16" => InnerType::Integer(IntegerInnerType::U16),
        "u32" => InnerType::Integer(IntegerInnerType::U32),
        "u64" => InnerType::Integer(IntegerInnerType::U64),
        "u128" => InnerType::Integer(IntegerInnerType::U128),
        "usize" => InnerType::Integer(IntegerInnerType::Usize),
        "i8" => InnerType::Integer(IntegerInnerType::I8),
        "i16" => InnerType::Integer(IntegerInnerType::I16),
        "i32" => InnerType::Integer(IntegerInnerType::I32),
        "i64" => InnerType::Integer(IntegerInnerType::I64),
        "i128" => InnerType::Integer(IntegerInnerType::I128),
        "isize" => InnerType::Integer(IntegerInnerType::Isize),
        "f32" => InnerType::Float(FloatInnerType::F32),
        "f64" => InnerType::Float(FloatInnerType::F64),
        "Decimal" | "rust_decimal::Decimal" | "::rust_decimal::Decimal" => {
            #[cfg(feature = "rust_decimal")]
            {
                InnerType::Decimal(crate::decimal::models::DecimalInnerType)
            }
            #[cfg(not(feature = "rust_decimal"))]
            {
                return Err(syn::Error::new(
                    seg.ty.span(),
                    concat!(
                        "To use `Decimal` as the inner type, enable the `rust_decimal` feature of crate `nutype`, e.g.\n\n",
                        "    nutype = { version = \"0.7\", features = [\"rust_decimal\"] }\n\n",
                        "You also need to add `rust_decimal` as a dependency of your crate."
                    ),
                ));
            }
        }
        _ => InnerType::Any(AnyInnerType::new(seg.clone())),
    };

    Ok(Meta {
        doc_attrs,
        type_name,
        generics,
        inner_type,
        vis,
        forwarded_attrs,
        inner_field_attrs,
        serde_customization,
    })
}

struct PartitionedStructAttrs {
    doc_attrs: Vec<Attribute>,
    forwarded_attrs: Vec<Attribute>,
}

fn attr_first_segment_is(attr: &Attribute, name: &str) -> bool {
    match attr.path().segments.first() {
        Some(path_segment) => path_segment.ident == name,
        None => false,
    }
}

/// Partition the attributes found on the struct itself:
/// * doc comments are kept separately (emitted first, as before);
/// * `#[derive(...)]` has been intercepted earlier with a targeted error;
/// * `#[serde(transparent)]` is consumed; any other serde key is rejected;
/// * `#[schemars(...)]` is rejected (nutype hand-writes JsonSchema);
/// * everything else is forwarded verbatim onto the generated struct.
///
/// Note: `#[cfg(...)]` never reaches this partition. rustc strips item-level
/// `cfg`/`cfg_attr` attributes before the attribute macro expands, so a
/// cfg'd-out nutype struct simply disappears (the desired semantics), no
/// matter whether the `#[cfg]` is written above or below `#[nutype]`.
fn partition_struct_attrs(
    attrs: Vec<Attribute>,
    serde_customization: &mut SerdeCustomization,
) -> Result<PartitionedStructAttrs, syn::Error> {
    let mut doc_attrs: Vec<Attribute> = Vec::new();
    let mut forwarded_attrs: Vec<Attribute> = Vec::new();

    for attr in attrs {
        if is_doc_attribute(&attr) {
            doc_attrs.push(attr);
        } else if is_derive_attribute(&attr) {
            // Already rejected by intercept_derive_macro(); unreachable here,
            // but keep the arm so the partition stays exhaustive in intent.
            continue;
        } else if attr_first_segment_is(&attr, "serde") {
            parse_struct_level_serde_attr(&attr, serde_customization)?;
        } else if attr_first_segment_is(&attr, "schemars") {
            let msg = "#[nutype] does not support `#[schemars(...)]` attributes.\n\
                 nutype generates its own implementation of `JsonSchema`, so there is no schemars derive that could read this attribute.";
            return Err(syn::Error::new(attr.span(), msg));
        } else {
            forwarded_attrs.push(attr);
        }
    }

    Ok(PartitionedStructAttrs {
        doc_attrs,
        forwarded_attrs,
    })
}

/// Parse a struct-level `#[serde(...)]` attribute.
/// The only supported key is `transparent`.
fn parse_struct_level_serde_attr(
    attr: &Attribute,
    serde_customization: &mut SerdeCustomization,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("transparent") {
            if serde_customization.transparent.is_some() {
                return Err(meta.error("Duplicated serde attribute `transparent`."));
            }
            serde_customization.transparent = Some(meta.path.span());
            Ok(())
        } else {
            Err(meta.error(
                "#[nutype] does not support this serde attribute on the type.\n\
                 The only supported type-level serde attribute is `#[serde(transparent)]`.\n\
                 To customize how the inner value is serialized, use the field-level attributes\n\
                 `#[serde(with = \"...\")]`, `#[serde(serialize_with = \"...\")]` or `#[serde(deserialize_with = \"...\")]`.",
            ))
        }
    })
}

/// Partition the attributes found on the inner field:
/// * `#[serde(...)]` is consumed (`with`, `serialize_with`, `deserialize_with`);
/// * `#[cfg(...)]` is rejected (it could cfg away the only field);
/// * everything else is forwarded verbatim onto the generated field.
fn partition_field_attrs(
    attrs: Vec<Attribute>,
    serde_customization: &mut SerdeCustomization,
) -> Result<Vec<Attribute>, syn::Error> {
    let mut forwarded: Vec<Attribute> = Vec::new();

    for attr in attrs {
        if attr_first_segment_is(&attr, "serde") {
            parse_field_level_serde_attr(&attr, serde_customization)?;
        } else if attr_first_segment_is(&attr, "cfg") {
            let msg = "#[nutype] does not support `#[cfg(...)]` on the inner field:\n\
                 it could remove the only field of the newtype and break the generated code.\n\
                 Note: `#[cfg_attr(...)]` is supported and forwarded as is.";
            return Err(syn::Error::new(attr.span(), msg));
        } else {
            forwarded.push(attr);
        }
    }

    Ok(forwarded)
}

/// Parse a field-level `#[serde(...)]` attribute.
/// Supported keys: `with = "module"`, `serialize_with = "path"`,
/// `deserialize_with = "path"` (string literals, like in serde itself).
fn parse_field_level_serde_attr(
    attr: &Attribute,
    serde_customization: &mut SerdeCustomization,
) -> Result<(), syn::Error> {
    fn parse_path_value(
        meta: &syn::meta::ParseNestedMeta,
    ) -> Result<SpannedItem<syn::Path>, syn::Error> {
        let lit: syn::LitStr = meta.value()?.parse()?;
        let path: syn::Path = lit.parse().map_err(|err| {
            let msg =
                format!("Expected a path (e.g. `my_module::my_function`), like in serde: {err}");
            syn::Error::new(lit.span(), msg)
        })?;
        Ok(SpannedItem::new(path, lit.span()))
    }

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("with") {
            if serde_customization.with.is_some() {
                return Err(meta.error("Duplicated serde attribute `with`."));
            }
            if serde_customization.serialize_with.is_some()
                || serde_customization.deserialize_with.is_some()
            {
                return Err(meta.error(
                    "`with` cannot be combined with `serialize_with` or `deserialize_with`.",
                ));
            }
            serde_customization.with = Some(parse_path_value(&meta)?);
            Ok(())
        } else if meta.path.is_ident("serialize_with") {
            if serde_customization.serialize_with.is_some() {
                return Err(meta.error("Duplicated serde attribute `serialize_with`."));
            }
            if serde_customization.with.is_some() {
                return Err(meta.error(
                    "`serialize_with` cannot be combined with `with`.",
                ));
            }
            serde_customization.serialize_with = Some(parse_path_value(&meta)?);
            Ok(())
        } else if meta.path.is_ident("deserialize_with") {
            if serde_customization.deserialize_with.is_some() {
                return Err(meta.error("Duplicated serde attribute `deserialize_with`."));
            }
            if serde_customization.with.is_some() {
                return Err(meta.error(
                    "`deserialize_with` cannot be combined with `with`.",
                ));
            }
            serde_customization.deserialize_with = Some(parse_path_value(&meta)?);
            Ok(())
        } else {
            Err(meta.error(
                "Unsupported serde attribute on the inner field.\n\
                 #[nutype] supports only `with`, `serialize_with` and `deserialize_with` here.\n\
                 If you need support for more, please open an issue: https://github.com/greyblake/nutype/issues",
            ))
        }
    })
}

fn validate_inner_field_visibility(vis: &Visibility) -> Result<(), syn::Error> {
    match vis {
        Visibility::Inherited => Ok(()),
        Visibility::Public(_) | Visibility::Restricted(_) => {
            let msg = "Oh, setting visibility for the inner field is forbidden by #[nutype].\nThe whole point is to guarantee that no value can be created without passing the guards (sanitizers and validators).\nWe do hope for your understanding and wish you a good sunny day!";
            Err(syn::Error::new(vis.span(), msg))
        }
    }
}
