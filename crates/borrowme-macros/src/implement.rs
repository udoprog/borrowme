use std::collections::HashSet;
use std::mem;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{token, Token};

use crate::attr;
use crate::ctxt::Ctxt;
use crate::respan::Respan;

const NAME: &str = "#[borrowme]";
const STATIC: &str = "static";
const STATIC_LT: &str = "'static";

#[derive(Debug, Clone, Copy)]
enum Access {
    SelfAccess,
    BindingAccess,
}

enum Binding {
    Named(syn::Ident),
    Unnamed(syn::Index),
}

impl Binding {
    fn as_member(&self) -> syn::Member {
        match self {
            Binding::Named(ident) => syn::Member::Named(ident.clone()),
            Binding::Unnamed(index) => syn::Member::Unnamed(index.clone()),
        }
    }

    /// Construct binding as a varaible name.
    fn as_variable(&self) -> syn::Ident {
        match self {
            Binding::Named(ident) => ident.clone(),
            Binding::Unnamed(index) => syn::Ident::new(&format!("f{}", index.index), index.span()),
        }
    }

    /// Construct `field: value` syntax.
    fn as_field_value(&self) -> syn::FieldValue {
        let member = self.as_member();

        match self {
            Binding::Named(ident) => syn::FieldValue {
                attrs: Vec::new(),
                member,
                colon_token: None,
                expr: syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: ident.clone().into(),
                }),
            },
            Binding::Unnamed(index) => {
                let ident = syn::Ident::new(&format!("f{}", index.index), index.span());

                syn::FieldValue {
                    attrs: Vec::new(),
                    member,
                    colon_token: Some(<Token![:]>::default()),
                    expr: syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: ident.into(),
                    }),
                }
            }
        }
    }
}

impl ToTokens for Binding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Binding::Named(field) => {
                field.to_tokens(tokens);
            }
            Binding::Unnamed(index) => {
                index.to_tokens(tokens);
            }
        }
    }
}

struct BoundAccess<'a> {
    copy: bool,
    access: Access,
    binding: &'a Binding,
}

impl BoundAccess<'_> {
    fn as_expr(&self) -> syn::Expr {
        match &self.access {
            Access::SelfAccess => {
                let expr = syn::Expr::Field(syn::ExprField {
                    attrs: Vec::new(),
                    base: Box::new(syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(<Token![self]>::default()),
                    })),
                    dot_token: <Token![.]>::default(),
                    member: self.binding.as_member(),
                });

                if self.copy {
                    return expr;
                }

                syn::Expr::Reference(syn::ExprReference {
                    attrs: Vec::new(),
                    and_token: <Token![&]>::default(),
                    mutability: None,
                    expr: Box::new(expr),
                })
            }
            Access::BindingAccess => syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: syn::Path::from(self.binding.as_variable()),
            }),
        }
    }
}

#[derive(Clone, Copy)]
enum Call<'a> {
    Path(&'a syn::Path),
    Ref,
}

impl Call<'_> {
    fn as_expr(self, access: &BoundAccess<'_>) -> syn::Expr {
        match self {
            Call::Path(path) => {
                let mut call = syn::ExprCall {
                    attrs: Vec::new(),
                    func: Box::new(syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: path.clone(),
                    })),
                    paren_token: token::Paren::default(),
                    args: Punctuated::new(),
                };

                call.args.push(access.as_expr());
                syn::Expr::Call(call)
            }
            Call::Ref => access.as_expr(),
        }
    }
}

pub(crate) fn implement(
    cx: &Ctxt,
    attrs: &[syn::Attribute],
    mut item: syn::Item,
) -> Result<TokenStream, ()> {
    let mut output = item.clone();

    let (to_owned_fn, borrow_fn) = match (&mut output, &mut item) {
        (syn::Item::Struct(o_st), syn::Item::Struct(b_st)) => {
            let attr = attr::container(cx, &o_st.ident, attrs, &o_st.attrs)?;
            attr::strip([&mut o_st.attrs, &mut b_st.attrs]);

            apply_attributes(&attr.attributes, &mut o_st.attrs, &mut b_st.attrs);
            strip_lifetimes(&mut o_st.generics);
            o_st.ident = attr.owned_ident;

            let mut to_owned_entries = Vec::new();
            let mut borrow_entries = Vec::new();

            process_fields(
                cx,
                Access::SelfAccess,
                &mut o_st.fields,
                &mut b_st.fields,
                &mut to_owned_entries,
                &mut borrow_entries,
            )?;

            let owned_ident = &o_st.ident;

            let to_owned_fn = quote! {
                #[inline]
                fn to_owned(&self) -> Self::Owned {
                    #owned_ident {
                        #(#to_owned_entries,)*
                    }
                }
            };

            let borrow_ident = &b_st.ident;

            let borrow_fn = quote! {
                #[inline]
                fn borrow(&self) -> Self::Target<'_> {
                    #borrow_ident {
                        #(#borrow_entries,)*
                    }
                }
            };

            (to_owned_fn, borrow_fn)
        }
        (syn::Item::Enum(o_en), syn::Item::Enum(b_en)) => {
            let attr = attr::container(cx, &o_en.ident, attrs, &o_en.attrs)?;
            attr::strip([&mut o_en.attrs, &mut b_en.attrs]);

            apply_attributes(&attr.attributes, &mut o_en.attrs, &mut b_en.attrs);
            strip_lifetimes(&mut o_en.generics);
            o_en.ident = attr.owned_ident;

            let mut to_owned_variants = Vec::new();
            let mut borrow_variants = Vec::new();

            let owned_ident = o_en.ident.clone();
            let borrow_ident = b_en.ident.clone();

            for (o_variant, b_variant) in o_en.variants.iter_mut().zip(b_en.variants.iter_mut()) {
                let attr = attr::variant(cx, &o_variant.attrs)?;
                attr::strip([&mut o_variant.attrs, &mut b_variant.attrs]);

                apply_attributes(&attr.attributes, &mut o_variant.attrs, &mut b_variant.attrs);

                let mut to_owned_entries = Vec::new();
                let mut borrow_entries = Vec::new();

                process_fields(
                    cx,
                    Access::BindingAccess,
                    &mut o_variant.fields,
                    &mut b_variant.fields,
                    &mut to_owned_entries,
                    &mut borrow_entries,
                )?;

                let fields = o_variant
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(n, f)| match &f.ident {
                        Some(ident) => Binding::Named(ident.clone()),
                        None => Binding::Unnamed(syn::Index::from(n)),
                    });

                let variant_ident = &o_variant.ident;
                let patterns = fields.clone().map(|b| b.as_field_value());

                to_owned_variants.push(quote! {
                    #borrow_ident::#variant_ident { #(#patterns,)* } => {
                        #owned_ident::#variant_ident {
                            #(#to_owned_entries,)*
                        }
                    }
                });

                let patterns = fields.clone().map(|b| b.as_field_value());

                borrow_variants.push(quote! {
                    #owned_ident::#variant_ident { #(#patterns,)* } => {
                        #borrow_ident::#variant_ident {
                            #(#borrow_entries,)*
                        }
                    }
                });
            }

            let to_owned_fn = quote! {
                #[inline]
                fn to_owned(&self) -> Self::Owned {
                    match self {
                        #(#to_owned_variants,)*
                    }
                }
            };

            let borrow_fn = quote! {
                #[inline]
                fn borrow(&self) -> Self::Target<'_> {
                    match self {
                        #(#borrow_variants,)*
                    }
                }
            };

            (to_owned_fn, borrow_fn)
        }
        (_, item) => {
            cx.span_error(
                item.span(),
                format_args!("{NAME}: is only supported on structs."),
            );
            return Err(());
        }
    };

    let (owned_ident, owned_generics) = match &output {
        syn::Item::Struct(st) => (&st.ident, &st.generics),
        syn::Item::Enum(en) => (&en.ident, &en.generics),
        _ => return Err(()),
    };

    let (borrow_ident, borrow_generics) = match &item {
        syn::Item::Struct(st) => (&st.ident, &st.generics),
        syn::Item::Enum(en) => (&en.ident, &en.generics),
        _ => {
            return Err(());
        }
    };

    let (_, to_owned_type_generics, _) = owned_generics.split_for_impl();

    let to_owned = {
        let (impl_generics, type_generics, where_generics) = borrow_generics.split_for_impl();
        let to_owned = &cx.borrowme_to_owned_t;

        quote! {
            #[automatically_derived]
            impl #impl_generics #to_owned for #borrow_ident #type_generics #where_generics {
                type Owned = #owned_ident #to_owned_type_generics;
                #to_owned_fn
            }
        }
    };

    let borrow = {
        let mut borrow_generics = borrow_generics.clone();

        // NB: Replace all borrowed lifetimes with `'this`, which borrows from
        // `&self` in `fn borrow`.
        let this_lt = syn::Lifetime::new("'this", Span::call_site());

        for g in &mut borrow_generics.params {
            if let syn::GenericParam::Lifetime(l) = g {
                l.lifetime = this_lt.clone();
            }
        }

        let (_, borrow_return_type_generics, _) = borrow_generics.split_for_impl();

        let (impl_generics, type_generics, where_generics) = owned_generics.split_for_impl();
        let owned_borrow = &cx.borrowme_borrow_t;

        quote! {
            #[automatically_derived]
            impl #impl_generics #owned_borrow for #owned_ident #type_generics #where_generics {
                type Target<#this_lt> = #borrow_ident #borrow_return_type_generics;
                #borrow_fn
            }
        }
    };

    let mut stream = TokenStream::new();
    item.to_tokens(&mut stream);
    output.to_tokens(&mut stream);
    to_owned.to_tokens(&mut stream);
    borrow.to_tokens(&mut stream);
    Ok(stream)
}

fn process_fields(
    cx: &Ctxt,
    access: Access,
    o_fields: &mut syn::Fields,
    b_fields: &mut syn::Fields,
    to_owned_entries: &mut Vec<syn::FieldValue>,
    borrow_entries: &mut Vec<syn::FieldValue>,
) -> Result<(), ()> {
    for (index, (o_field, b_field)) in o_fields.iter_mut().zip(b_fields.iter_mut()).enumerate() {
        let field_ty_spans = field_ty_spans(o_field);

        let mut attr = attr::field(cx, field_ty_spans, &o_field.attrs)?;
        attr::strip([&mut o_field.attrs, &mut b_field.attrs]);
        apply_attributes(&attr.attributes, &mut o_field.attrs, &mut b_field.attrs);

        // Ensure that the field does not make use of any lifetimes.
        let ignore = HashSet::new();
        let mut lifetimes = Vec::new();
        let mut as_ty = o_field.ty.clone();

        let (type_hint, reference_type) = process_type(&mut as_ty, &ignore, &mut lifetimes);

        // Provide diagnostics in case there are field lifetimes we can't
        // make anything out of. Such as a `&'a str` field marked with
        // `#[copy]`.
        match attr.ty.kind {
            attr::FieldTypeKind::Copy(true) => {
                for (span, lt) in lifetimes {
                    let mut error = if lt.is_some() {
                        syn::Error::new(span, format_args!("{NAME}: lifetime not supported."))
                    } else {
                        syn::Error::new(
                            span,
                            format_args!("{NAME}: anonymous references not supported."),
                        )
                    };

                    error.combine(syn::Error::new(
                        o_field.span(),
                        "Hint: add #[owned(ty = <type>)] to specify which type to override this field with",
                    ));
                    cx.error(error);
                }
            }
            _ => {
                let is_std_ref =
                    matches!(attr.ty.kind, attr::FieldTypeKind::Std if reference_type.is_some());

                // For non-copy types, build an expression that tries to use the
                // `ToOwned` implementation to figure out which type to use.
                match type_hint {
                    TypeHint::None if attr.ty.owned.is_none() && !is_std_ref => {
                        let mut path = cx.borrowme_to_owned_t.clone();

                        path.segments.push(syn::PathSegment::from(syn::Ident::new(
                            "Owned",
                            Span::call_site(),
                        )));

                        let ty = syn::Type::Path(syn::TypePath {
                            qself: Some(syn::QSelf {
                                lt_token: <Token![<]>::default(),
                                ty: Box::new(as_ty),
                                position: 2,
                                as_token: Some(<Token![as]>::default()),
                                gt_token: <Token![>]>::default(),
                            }),
                            path,
                        });

                        attr.ty.owned = Some(Respan::new(ty, field_ty_spans));
                    }
                    TypeHint::Copy => {
                        if !matches!(attr.ty.kind, attr::FieldTypeKind::Copy(false)) {
                            attr.ty.kind = attr::FieldTypeKind::Copy(true);
                        }
                    }
                    _ => {}
                }
            }
        };

        let (to_owned, borrow) = match (attr.ty.kind, reference_type, attr.ty.owned) {
            (attr::FieldTypeKind::Copy(true), _, _) => (Call::Ref, Call::Ref),
            (attr::FieldTypeKind::Std, _, Some(ty)) => {
                o_field.ty = ty.into_type();
                (Call::Path(&cx.clone_t_clone), Call::Ref)
            }
            (attr::FieldTypeKind::Std, Some(ty), None) => {
                o_field.ty = ty;
                (Call::Path(&cx.clone_t_clone), Call::Ref)
            }
            (_, _, Some(ty)) => {
                o_field.ty = ty.into_type();
                (Call::Path(&attr.to_owned), Call::Path(&attr.borrow))
            }
            _ => {
                let clone = &cx.clone_t_clone;
                (Call::Path(clone), Call::Path(clone))
            }
        };

        let binding = match &o_field.ident {
            Some(ident) => Binding::Named(ident.clone()),
            None => Binding::Unnamed(syn::Index::from(index)),
        };

        let member = binding.as_member();

        let bound = BoundAccess {
            copy: matches!(attr.ty.kind, attr::FieldTypeKind::Copy(true)),
            access,
            binding: &binding,
        };

        to_owned_entries.push(syn::FieldValue {
            attrs: Vec::new(),
            member: member.clone(),
            colon_token: Some(<Token![:]>::default()),
            expr: to_owned.as_expr(&bound),
        });

        borrow_entries.push(syn::FieldValue {
            attrs: Vec::new(),
            member,
            colon_token: Some(<Token![:]>::default()),
            expr: borrow.as_expr(&bound),
        });
    }

    Ok(())
}

/// Calculate the field type span to use for diagnostics such as when there is a
/// type mismatch.
fn field_ty_spans(field: &syn::Field) -> (Span, Span) {
    let start = field.ty.span();
    let end = end_span(&field.ty).unwrap_or(start);
    (start, end)
}

/// Calculate the end span to use for a token stream.
fn end_span<T>(tokens: &T) -> Option<Span>
where
    T: ToTokens,
{
    Some(tokens.to_token_stream().into_iter().last()?.span())
}

/// Apply attributes to the appropriate variant.
fn apply_attributes(
    attributes: &attr::Attributes,
    owned_attrs: &mut Vec<syn::Attribute>,
    borrowed_attrs: &mut Vec<syn::Attribute>,
) {
    for meta in &attributes.own {
        owned_attrs.push(syn::Attribute {
            pound_token: <Token![#]>::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: token::Bracket::default(),
            meta: meta.clone(),
        });
    }

    for meta in &attributes.borrow {
        borrowed_attrs.push(syn::Attribute {
            pound_token: <Token![#]>::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: token::Bracket::default(),
            meta: meta.clone(),
        });
    }
}

#[derive(Debug, Clone, Copy)]
enum TypeHint {
    /// No particular type hint.
    None,
    /// Type looks like it could be copy, such as `'static T`.
    Copy,
}

impl TypeHint {
    /// Combine this type hint with another.
    fn combine(&mut self, other: TypeHint) {
        *self = match (*self, other) {
            (TypeHint::Copy, TypeHint::Copy) => TypeHint::Copy,
            (_, other) => other,
        };
    }
}

/// Capture all lifetimes from a type, and return a type which has all lifetimes
/// sanitized so it can be used in a `<<$ty> as ToOwned>::Owned` expression.
fn process_type<'ty>(
    ty: &mut syn::Type,
    ignore: &HashSet<syn::Ident>,
    out: &mut Vec<(Span, Option<syn::Lifetime>)>,
) -> (TypeHint, Option<syn::Type>) {
    match ty {
        syn::Type::Array(ty) => {
            let (hint, _) = process_type(&mut ty.elem, ignore, out);
            (hint, None)
        }
        syn::Type::BareFn(ty) => {
            let mut ignore = ignore.clone();

            // ignore for <'a, 'b, 'c> lifetimes
            if let Some(bound) = &ty.lifetimes {
                for param in &bound.lifetimes {
                    if let syn::GenericParam::Lifetime(lt) = param {
                        ignore.insert(lt.lifetime.ident.clone());
                    }
                }
            }

            for arg in &mut ty.inputs {
                process_type(&mut arg.ty, &ignore, out);
            }

            // NB: bare function are copy.
            (TypeHint::Copy, None)
        }
        syn::Type::Group(ty) => process_type(&mut ty.elem, ignore, out),
        syn::Type::Reference(ty) => {
            if let Some(lt) = &ty.lifetime {
                if ignore.contains(&lt.ident) || lt.ident == STATIC {
                    return (TypeHint::Copy, None);
                }
            }

            let span = ty
                .lifetime
                .as_ref()
                .map(|lt| lt.span())
                .unwrap_or_else(|| ty.and_token.span());

            // NB: We replace this with the static lifetime to *aid* type
            // inference, because the `ToOwned::Owned` variant will be the same
            // regardless.
            out.push((
                span,
                ty.lifetime.replace(syn::Lifetime::new(STATIC_LT, span)),
            ));
            (TypeHint::None, Some((*ty.elem).clone()))
        }
        syn::Type::Slice(ty) => {
            process_type(&mut ty.elem, ignore, out);
            // Slice types such as [T] are not copy, and they do in fact
            // indicate that the container is unsized.
            (TypeHint::None, None)
        }
        syn::Type::Tuple(ty) => {
            let mut hint = TypeHint::Copy;

            for ty in &mut ty.elems {
                hint.combine(process_type(ty, ignore, out).0);
            }

            (hint, None)
        }
        syn::Type::Path(ty) => {
            if let Some(ident) = &ty.path.get_ident() {
                let ident = ident.to_string();

                // NB: Primitive-looking types. This can fail at which point the
                // user is required to specify `#[no_copy]`.
                match ident.as_str() {
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => {
                        return (TypeHint::Copy, None)
                    }
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => {
                        return (TypeHint::Copy, None)
                    }
                    "f32" | "f64" => return (TypeHint::Copy, None),
                    "bool" => return (TypeHint::Copy, None),
                    _ => {}
                }
            }

            for s in &mut ty.path.segments {
                match &mut s.arguments {
                    syn::PathArguments::AngleBracketed(generics) => {
                        process_generic_type(&mut generics.args, ignore, out);
                    }
                    syn::PathArguments::Parenthesized(generics) => {
                        for ty in &mut generics.inputs {
                            process_type(ty, ignore, out);
                        }
                    }
                    _ => {}
                }
            }

            // NB: Since we can't peek through into the implementation of a path
            // argument, we can't make any assumptions about if they are `Copy`
            // or not. Even though types such as `Option<&'static str>` are
            // `Copy`.
            (TypeHint::None, None)
        }
        _ => (TypeHint::None, None),
    }
}

fn process_generic_type<'ty, P>(
    generics: &mut Punctuated<syn::GenericArgument, P>,
    ignore: &HashSet<syn::Ident>,
    out: &mut Vec<(Span, Option<syn::Lifetime>)>,
) {
    for argument in generics.iter_mut() {
        match argument {
            syn::GenericArgument::Lifetime(lt) => {
                // Don't touch existing static lifetimes.
                if lt.ident == STATIC {
                    return;
                }

                // NB: We replace this with the static lifetime to *aid* type
                // inference, because the `ToOwned::Owned` variant will be the
                // same regardless.
                out.push((
                    lt.span(),
                    Some(mem::replace(lt, syn::Lifetime::new(STATIC_LT, lt.span()))),
                ));
            }
            syn::GenericArgument::Type(ty) => {
                process_type(ty, ignore, out);
            }
            _ => {}
        }
    }
}

/// Strip lifetime parameters from the given generics.
fn strip_lifetimes(generics: &mut syn::Generics) {
    let mut params = generics.params.clone();
    params.clear();

    for p in &generics.params {
        if !matches!(p, syn::GenericParam::Lifetime(..)) {
            params.push(p.clone());
        }
    }

    generics.params = params;
}
