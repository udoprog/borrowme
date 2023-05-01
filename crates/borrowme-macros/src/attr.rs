use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::Token;

use crate::ctxt::Ctxt;
use crate::respan::Respan;

pub(crate) const COPY: &str = "copy";
pub(crate) const NO_COPY: &str = "no_copy";
pub(crate) const BORROWME: &str = "borrowme";
pub(crate) const BORROWED_ATTR: &str = "borrowed_attr";
pub(crate) const OWNED_ATTR: &str = "owned_attr";
pub(crate) const OWNED: &str = "owned";

const STRIP: [&str; 6] = [COPY, NO_COPY, BORROWED_ATTR, OWNED_ATTR, BORROWME, OWNED];

#[derive(Default)]
pub(crate) struct Attributes {
    // Attributes to only include on the owned variant.
    pub(crate) own: Vec<syn::Meta>,
    // Attributes to only include on the borrowed variant.
    pub(crate) borrow: Vec<syn::Meta>,
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
    attrs: &[syn::Attribute],
    rest: &[syn::Attribute],
) -> Result<Container, ()> {
    let mut attr = Container {
        owned_ident: quote::format_ident!("Owned{}", ident),
        attributes: Attributes::default(),
    };

    for a in attrs.iter().chain(rest) {
        let result = if a.path().is_ident(BORROWME) {
            a.parse_nested_meta(|meta| {
                if meta.path.is_ident("prefix") {
                    meta.input.parse::<Token![=]>()?;
                    let prefix: syn::Ident = meta.input.parse()?;
                    attr.owned_ident = quote::format_ident!("{prefix}{}", ident);
                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{BORROWME}]: Unsupported attribute"),
                ))
            })
        } else if a.path().is_ident(BORROWED_ATTR) {
            a.parse_args_with(|input: ParseStream<'_>| {
                attr.attributes.borrow.push(input.parse()?);
                Ok(())
            })
        } else if a.path().is_ident(OWNED_ATTR) {
            a.parse_args_with(|input: ParseStream<'_>| {
                attr.attributes.own.push(input.parse()?);
                Ok(())
            })
        } else {
            continue;
        };

        if let Err(error) = result {
            cx.error(error);
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
        let result = if a.path().is_ident(BORROWME) {
            a.parse_nested_meta(|meta| {
                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{BORROWME}]: Unsupported attribute"),
                ))
            })
        } else if a.path().is_ident(BORROWED_ATTR) {
            a.parse_args_with(|input: ParseStream<'_>| {
                variant.attributes.borrow.push(input.parse()?);
                Ok(())
            })
        } else if a.path().is_ident(OWNED_ATTR) {
            a.parse_args_with(|input: ParseStream<'_>| {
                variant.attributes.own.push(input.parse()?);
                Ok(())
            })
        } else {
            continue;
        };

        if let Err(error) = result {
            cx.error(error);
        }
    }

    Ok(variant)
}

#[derive(Default, Debug, Clone, Copy)]
pub(crate) enum FieldTypeKind {
    // Clone the original field.
    #[default]
    Default,
    // Explicit indication if the field is copy.
    Copy(bool),
    // Explicitly std traits to handle the field.
    Std,
}

#[derive(Default)]
pub(crate) struct FieldType {
    pub(crate) kind: FieldTypeKind,
    pub(crate) owned: Option<Respan<syn::Type>>
}

pub(crate) struct Field {
    // Replace the type of the field.
    pub(crate) ty: FieldType,
    pub(crate) borrow: syn::Path,
    pub(crate) to_owned: syn::Path,
    pub(crate) attributes: Attributes,
}

/// Parse field attributes.
///
/// We provide `field_spans` so that the processed `FieldType::Type` can be
/// respanned to emit better diagnostics in case if fails something like a type
/// check.
pub(crate) fn field(cx: &Ctxt, spans: (Span, Span), attrs: &[syn::Attribute]) -> Result<Field, ()> {
    let mut field = Field {
        ty: FieldType::default(),
        borrow: cx.borrowme_borrow_t_borrow.clone(),
        to_owned: cx.borrowme_to_owned_t_borrow.clone(),
        attributes: Attributes::default(),
    };

    for a in attrs {
        let result = if a.path().is_ident(COPY) {
            if matches!(&a.meta, syn::Meta::Path(..)) {
                field.ty.kind = FieldTypeKind::Copy(true);
                Ok(())
            } else {
                Err(syn::Error::new(
                    a.span(),
                    format_args!("#[{COPY}] Expected no arguments"),
                ))
            }
        } else if a.path().is_ident(NO_COPY) {
            if matches!(&a.meta, syn::Meta::Path(..)) {
                field.ty.kind = FieldTypeKind::Copy(false);
                Ok(())
            } else {
                Err(syn::Error::new(
                    a.span(),
                    format_args!("#[{NO_COPY}] Expected no arguments"),
                ))
            }
        } else if a.path().is_ident(OWNED) {
            a.parse_args_with(|input: ParseStream<'_>| {
                field.ty.owned = Some(Respan::new(input.parse()?, spans));
                Ok(())
            })
        } else if a.path().is_ident(BORROWME) {
            a.parse_nested_meta(|meta| {
                if meta.path.is_ident(OWNED) {
                    meta.input.parse::<Token![=]>()?;
                    field.ty.owned = Some(Respan::new(meta.input.parse()?, spans));
                    return Ok(());
                }

                if meta.path.is_ident(COPY) {
                    field.ty.kind = FieldTypeKind::Copy(true);
                    return Ok(());
                }

                if meta.path.is_ident(NO_COPY) {
                    field.ty.kind = FieldTypeKind::Copy(false);
                    return Ok(());
                }

                if meta.path.is_ident("std") {
                    field.ty.kind = FieldTypeKind::Std;
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

                Err(syn::Error::new(
                    meta.path.span(),
                    format_args!("#[{BORROWME}]: Unsupported attribute"),
                ))
            })
        } else if a.path().is_ident(BORROWED_ATTR) {
            a.parse_args_with(|input: ParseStream<'_>| {
                field.attributes.borrow.push(input.parse()?);
                Ok(())
            })
        } else if a.path().is_ident(OWNED_ATTR) {
            a.parse_args_with(|input: ParseStream<'_>| {
                field.attributes.own.push(input.parse()?);
                Ok(())
            })
        } else {
            continue;
        };

        if let Err(error) = result {
            cx.error(error);
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
        attrs.retain(|a| STRIP.iter().all(|name| !a.path().is_ident(name)));
    }
}
