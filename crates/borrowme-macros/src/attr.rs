use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::Token;

use crate::ctxt::Ctxt;

/// The name of the attribute being processed.
pub(crate) const OWNED: &str = "owned";
/// The name of the attribute being processed.
pub(crate) const BORROWED: &str = "borrowed";

#[derive(Default)]
pub(crate) struct Attributes {
    // Attributes to only include on the owned variant.
    pub(crate) owned: Option<syn::Meta>,
    // Attributes to only include on the borrowed variant.
    pub(crate) borrowed: Option<syn::Meta>,
}

/// Container attributes.
pub(crate) struct Container {
    // The name of the container.
    pub(crate) owned_ident: syn::Ident,
    // Attributes to apply.
    pub(crate) attributes: Attributes,
}

/// Parse container attributes.
pub(crate) fn container(
    cx: &Ctxt,
    ident: &syn::Ident,
    attrs: &Vec<syn::Attribute>,
) -> Result<Container, ()> {
    let mut attr = Container {
        owned_ident: quote::format_ident!("Owned{}", ident),
        attributes: Attributes::default(),
    };

    for a in attrs {
        if a.path().is_ident(OWNED) {
            let result = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("prefix") {
                    meta.input.parse::<Token![=]>()?;
                    let prefix: syn::Ident = meta.input.parse()?;
                    attr.owned_ident = quote::format_ident!("{prefix}{}", ident);
                    return Ok(());
                }

                if meta.path.is_ident("attr") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    attr.attributes.owned = Some(content.parse()?);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{OWNED}]: Unsupported attribute"),
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        } else if a.path().is_ident(BORROWED) {
            let result = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("attr") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    attr.attributes.borrowed = Some(content.parse()?);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{BORROWED}]: Unsupported attribute"),
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    Ok(attr)
}

pub(crate) struct Variant {
    pub(crate) attributes: Attributes,
}

/// Parse variant attributes.
pub(crate) fn variant(cx: &Ctxt, attrs: &[syn::Attribute]) -> Result<Variant, ()> {
    let mut variant = Variant {
        attributes: Attributes::default(),
    };

    for a in attrs {
        if a.path().is_ident(OWNED) {
            let result = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("attr") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    variant.attributes.owned = Some(content.parse()?);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{OWNED}]: Unsupported attribute"),
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        } else if a.path().is_ident(BORROWED) {
            let result = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("attr") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    variant.attributes.borrowed = Some(content.parse()?);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{BORROWED}]: Unsupported attribute"),
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    Ok(variant)
}

#[derive(Default)]
pub(crate) enum FieldType {
    // Clone the original field.
    #[default]
    Original,
    // Copy the original field.
    Copy,
    // Replace with type.
    Type(syn::Type),
}

pub(crate) struct Field {
    // Replace the type of the field.
    pub(crate) ty: FieldType,
    pub(crate) borrow: syn::Path,
    pub(crate) to_owned: syn::Path,
    pub(crate) attributes: Attributes,
}

/// Parse field attributes.
pub(crate) fn field(cx: &Ctxt, attrs: &[syn::Attribute]) -> Result<Field, ()> {
    let mut field = Field {
        ty: FieldType::default(),
        borrow: cx.borrow.clone(),
        to_owned: cx.to_owned.clone(),
        attributes: Attributes::default(),
    };

    for a in attrs {
        if a.path().is_ident(OWNED) {
            let result = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("ty") {
                    meta.input.parse::<Token![=]>()?;
                    field.ty = FieldType::Type(meta.input.parse()?);
                    return Ok(());
                }

                if meta.path.is_ident("to_owned_with") {
                    let (path, _) = parse_path(&meta)?;
                    field.to_owned = path;
                    return Ok(());
                }

                if meta.path.is_ident("borrow_with") {
                    let (path, _) = parse_path(&meta)?;
                    field.borrow = path;
                    return Ok(());
                }

                if meta.path.is_ident("with") {
                    let (path, span) = parse_path(&meta)?;

                    field.to_owned = path.clone();
                    field.to_owned.segments.push(syn::PathSegment {
                        ident: syn::Ident::new("to_owned", span),
                        arguments: syn::PathArguments::None,
                    });

                    field.borrow = path;
                    field.borrow.segments.push(syn::PathSegment {
                        ident: syn::Ident::new("borrow", span),
                        arguments: syn::PathArguments::None,
                    });

                    return Ok(());
                }

                if meta.path.is_ident("copy") {
                    field.ty = FieldType::Copy;
                    return Ok(());
                }

                if meta.path.is_ident("attr") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    field.attributes.owned = Some(content.parse()?);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{OWNED}]: Unsupported attribute"),
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        } else if a.path().is_ident(BORROWED) {
            let result = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("attr") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    field.attributes.borrowed = Some(content.parse()?);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{BORROWED}]: Unsupported attribute"),
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    Ok(field)
}

fn parse_path(meta: &ParseNestedMeta) -> syn::Result<(syn::Path, proc_macro2::Span)> {
    meta.input.parse::<Token![=]>()?;

    let path: syn::Path = meta.input.parse()?;

    let last = path
        .segments
        .last()
        .map(|l| l.span())
        .unwrap_or(path.span());

    Ok((path, last))
}

pub(crate) fn strip<const N: usize>(attrs: [&mut Vec<syn::Attribute>; N]) {
    for attrs in attrs {
        attrs.retain(|a| !a.path().is_ident(OWNED) && !a.path().is_ident(BORROWED));
    }
}
