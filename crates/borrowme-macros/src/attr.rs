use std::fmt;

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
    /// Attributes to only include on the owned variant.
    pub(crate) own: Vec<syn::Meta>,
    /// Attributes to only include on the borrowed variant.
    pub(crate) borrow: Vec<syn::Meta>,
}

/// Container attributes.
pub(crate) struct Container {
    /// The name of the container.
    pub(crate) owned_ident: Option<(Span, syn::Ident)>,
    /// Attributes to apply.
    pub(crate) attributes: Attributes,
    /// Default field type kind.
    pub(crate) kind: Option<(Span, FieldTypeKind)>,
}

impl Container {
    pub(crate) fn owned_ident(&self, ident: &syn::Ident) -> syn::Ident {
        if let Some((_, ident)) = &self.owned_ident {
            ident.clone()
        } else {
            quote::format_ident!("Owned{}", ident)
        }
    }
}

/// Parse container attributes.
pub(crate) fn container(
    cx: &Ctxt,
    attrs: &[syn::Attribute],
    rest: &[syn::Attribute],
) -> Result<Container, ()> {
    let mut attr = Container {
        owned_ident: None,
        attributes: Attributes::default(),
        kind: None,
    };

    macro_rules! set_attr {
        ($field:ident $(. $field2:ident)*, $meta:expr, $value:expr, $message:expr $(,)?) => {
            set_attr(cx, &mut attr.$field$(.$field2)*, $meta, $value, $message)
        }
    }

    for a in attrs.iter().chain(rest) {
        let result = if a.path().is_ident(BORROWME) {
            a.parse_nested_meta(|meta| {
                let span = meta.path.span();

                if meta.path.is_ident("name") {
                    meta.input.parse::<Token![=]>()?;
                    set_attr!(owned_ident, span, meta.input.parse()?, "Duplicate name.",);
                    return Ok(());
                }

                if meta.path.is_ident("std") {
                    let kind = FieldTypeKind::Std;
                    set_attr!(kind, span, kind, "Duplicate container field kind.");
                    return Ok(());
                }

                Err(syn::Error::new(
                    span,
                    format_args!("#[{BORROWME}]: Unsupported attribute."),
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
    pub(crate) kind: Option<(Span, FieldTypeKind)>,
}

/// Parse variant attributes.
pub(crate) fn variant(
    cx: &Ctxt,
    attrs: &[syn::Attribute],
    container: &Container,
) -> Result<Variant, ()> {
    let mut variant = Variant {
        attributes: Attributes::default(),
        kind: None,
    };

    macro_rules! set_attr {
        ($field:ident $(. $field2:ident)*, $meta:expr, $value:expr, $message:expr $(,)?) => {
            set_attr(cx, &mut variant.$field$(.$field2)*, $meta, $value, $message)
        }
    }

    for a in attrs {
        let result = if a.path().is_ident(BORROWME) {
            a.parse_nested_meta(|meta| {
                let span = meta.path.span();

                if meta.path.is_ident("std") {
                    let kind = FieldTypeKind::Std;
                    set_attr!(kind, span, kind, "Duplicate variant field kind.");
                    return Ok(());
                }

                Err(syn::Error::new(
                    span,
                    format_args!("#[{BORROWME}]: Unsupported attribute."),
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

    if variant.kind.is_none() {
        variant.kind = container.kind;
    }

    Ok(variant)
}

#[derive(Default, Debug, Clone, Copy)]
pub(crate) enum FieldTypeKind {
    /// Clone the original field.
    #[default]
    Default,
    /// Explicit indication if the field is copy.
    Copy(bool),
    /// Explicitly std traits to handle the field.
    Std,
}

#[derive(Default)]
pub(crate) struct FieldType {
    pub(crate) kind: Option<(Span, FieldTypeKind)>,
    pub(crate) owned: Option<(Span, Respan<syn::Type>)>,
}

impl FieldType {
    pub(crate) fn kind(&self) -> FieldTypeKind {
        self.kind.as_ref().map(|(_, v)| *v).unwrap_or_default()
    }

    pub(crate) fn set_kind(&mut self, kind: FieldTypeKind) {
        self.kind = Some((Span::call_site(), kind));
    }

    pub(crate) fn owned(&self) -> Option<&Respan<syn::Type>> {
        Some(&self.owned.as_ref()?.1)
    }

    pub(crate) fn set_owned(&mut self, owned: Respan<syn::Type>) {
        self.owned = Some((Span::call_site(), owned));
    }
}

pub(crate) struct Field {
    /// Whether the field needs mut or not.
    pub(crate) is_mut: Option<(Span, ())>,
    /// Replace the type of the field.
    pub(crate) ty: FieldType,
    pub(crate) borrow: Option<(Span, syn::Path)>,
    pub(crate) borrow_mut: Option<(Span, syn::Path)>,
    pub(crate) to_owned: Option<(Span, syn::Path)>,
    pub(crate) attributes: Attributes,
}

impl Field {
    /// Get borrow implementation.
    pub(crate) fn borrow<'a>(&'a self, cx: &'a Ctxt) -> &'a syn::Path {
        self.borrow
            .as_ref()
            .map(|(_, p)| p)
            .unwrap_or(&cx.borrowme_borrow_t_borrow)
    }

    /// Get borrow_mut implementation.
    pub(crate) fn borrow_mut<'a>(&'a self, cx: &'a Ctxt) -> &'a syn::Path {
        self.borrow_mut
            .as_ref()
            .map(|(_, p)| p)
            .unwrap_or(&cx.borrowme_borrow_mut_t_borrow_mut)
    }

    /// Get to_owned implementation.
    pub(crate) fn to_owned<'a>(&'a self, cx: &'a Ctxt) -> &'a syn::Path {
        self.to_owned
            .as_ref()
            .map(|(_, p)| p)
            .unwrap_or(&cx.borrowme_to_owned_t_to_owned)
    }

    /// Test if field is mutable.
    pub(crate) fn is_mut(&self) -> bool {
        self.is_mut.is_some()
    }
}

/// Parse field attributes.
///
/// We provide `field_spans` so that the processed `FieldType::Type` can be
/// respanned to emit better diagnostics in case if fails something like a type
/// check.
pub(crate) fn field(
    cx: &Ctxt,
    spans: (Span, Span),
    attrs: &[syn::Attribute],
    default_kind: Option<(Span, FieldTypeKind)>,
) -> Result<Field, ()> {
    let mut attr = Field {
        is_mut: None,
        ty: FieldType::default(),
        borrow: None,
        borrow_mut: None,
        to_owned: None,
        attributes: Attributes::default(),
    };

    macro_rules! set_attr {
        ($field:ident $(. $field2:ident)*, $meta:expr, $value:expr, $message:expr) => {
            set_attr(cx, &mut attr.$field$(.$field2)*, $meta, $value, $message)
        }
    }

    for a in attrs {
        let result = if a.path().is_ident(COPY) {
            if matches!(&a.meta, syn::Meta::Path(..)) {
                set_attr!(
                    ty.kind,
                    a.path().span(),
                    FieldTypeKind::Copy(true),
                    "Duplicate field kind from copy attribute."
                );
                Ok(())
            } else {
                Err(syn::Error::new(
                    a.span(),
                    format_args!("#[{COPY}] Expected no arguments."),
                ))
            }
        } else if a.path().is_ident(NO_COPY) {
            if matches!(&a.meta, syn::Meta::Path(..)) {
                set_attr!(
                    ty.kind,
                    a.path().span(),
                    FieldTypeKind::Copy(false),
                    "Duplicate field kind from no_copy attribute."
                );
                Ok(())
            } else {
                Err(syn::Error::new(
                    a.span(),
                    format_args!("#[{NO_COPY}] Expected no arguments."),
                ))
            }
        } else if a.path().is_ident(OWNED) {
            a.parse_args_with(|input: ParseStream<'_>| {
                set_attr!(
                    ty.owned,
                    a.path().span(),
                    Respan::new(input.parse()?, spans),
                    "Duplicate owned attribute."
                );
                Ok(())
            })
        } else if a.path().is_ident(BORROWME) {
            a.parse_nested_meta(|meta| {
                let span = meta.path.span();

                if meta.path.is_ident(OWNED) {
                    meta.input.parse::<Token![=]>()?;
                    let ty = Respan::new(meta.input.parse()?, spans);
                    set_attr!(ty.owned, span, ty, "Duplicate owned attribute.");
                    return Ok(());
                }

                if meta.path.is_ident("mut") {
                    set_attr!(is_mut, span, (), "Duplicate attribute setting mutability.");
                    return Ok(());
                }

                if meta.path.is_ident(COPY) {
                    let kind = FieldTypeKind::Copy(false);
                    set_attr!(ty.kind, span, kind, "Duplicate field kind.");
                    return Ok(());
                }

                if meta.path.is_ident(NO_COPY) {
                    let kind = FieldTypeKind::Copy(false);
                    set_attr!(ty.kind, span, kind, "Duplicate field kind.");
                    return Ok(());
                }

                if meta.path.is_ident("std") {
                    let kind = FieldTypeKind::Std;
                    set_attr!(ty.kind, span, kind, "Duplicate field kind.");
                    return Ok(());
                }

                if meta.path.is_ident("to_owned_with") {
                    let (path, _) = parse_path(&meta)?;
                    set_attr!(to_owned, span, path, "Duplicate to_owned_with.");
                    return Ok(());
                }

                if meta.path.is_ident("borrow_with") {
                    let (path, _) = parse_path(&meta)?;
                    set_attr!(borrow, span, path, "Duplicate borrow_with.");
                    return Ok(());
                }

                if meta.path.is_ident("borrow_mut_with") {
                    let (path, _) = parse_path(&meta)?;
                    set_attr!(borrow_mut, span, path, "Duplicate borrow_mut_with.");
                    set_attr!(is_mut, span, (), "Duplicate attribute setting mutability.");
                    return Ok(());
                }

                if meta.path.is_ident("with") {
                    let (path, span) = parse_path(&meta)?;

                    let mut to_owned = path.clone();
                    to_owned
                        .segments
                        .push(syn::PathSegment::from(syn::Ident::new("to_owned", span)));
                    set_attr!(to_owned, span, to_owned, "Duplicate to_owned_with.");

                    let mut borrow = path.clone();
                    borrow
                        .segments
                        .push(syn::PathSegment::from(syn::Ident::new("borrow", span)));
                    set_attr!(borrow, span, borrow, "Duplicate borrow_with.");

                    let mut borrow_mut = path;
                    borrow_mut
                        .segments
                        .push(syn::PathSegment::from(syn::Ident::new("borrow_mut", span)));
                    set_attr!(borrow_mut, span, borrow_mut, "Duplicate borrow_mut_with.");
                    return Ok(());
                }

                Err(syn::Error::new(
                    span,
                    format_args!("#[{BORROWME}]: Unsupported attribute."),
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

    if attr.ty.kind.is_none() {
        attr.ty.kind = default_kind;
    }

    Ok(attr)
}

fn set_attr<T>(
    cx: &Ctxt,
    existing: &mut Option<(Span, T)>,
    meta: Span,
    value: T,
    error: impl fmt::Display,
) {
    if let Some((span, _)) = existing.as_ref() {
        cx.span_error(meta, format_args!("#[{BORROWME}] {error}"));
        cx.span_error(*span, format_args!("#[{BORROWME}] Existing one is here."));
    } else {
        *existing = Some((meta, value));
    }
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
