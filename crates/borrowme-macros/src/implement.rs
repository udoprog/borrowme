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
    use_reference: bool,
    is_mut: bool,
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

                if !self.use_reference {
                    return expr;
                }

                syn::Expr::Reference(syn::ExprReference {
                    attrs: Vec::new(),
                    and_token: <Token![&]>::default(),
                    mutability: self.is_mut.then(<Token![mut]>::default),
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

    let mut needs_mut = false;
    let mut clone_type_params_out: HashSet<syn::Ident> = HashSet::new();
    let mut borrowed_type_params_out: HashSet<syn::Ident> = HashSet::new();

    let (to_owned_fn, borrow_fn) = match (&mut output, &mut item) {
        (syn::Item::Struct(o_st), syn::Item::Struct(b_st)) => {
            let attr = attr::container(cx, attrs, &o_st.attrs)?;
            attr::strip([&mut o_st.attrs, &mut b_st.attrs]);

            apply_attributes(&attr.attributes, &mut o_st.attrs, &mut b_st.attrs);
            process_generics(
                cx,
                o_st.ident.span(),
                &mut o_st.generics,
                o_st.fields.is_empty(),
            );
            o_st.ident = attr.owned_ident(&o_st.ident);

            let mut to_owned_entries = Vec::new();
            let mut borrow_entries = Vec::new();
            let type_params = collect_type_param_idents(&b_st.generics);
            let mut clone_type_params = HashSet::new();
            let mut borrowed_type_params = HashSet::new();

            process_fields(
                cx,
                Access::SelfAccess,
                attr.kind,
                &mut o_st.fields,
                &mut b_st.fields,
                &mut to_owned_entries,
                &mut borrow_entries,
                &mut needs_mut,
                &type_params,
                &mut clone_type_params,
                &mut borrowed_type_params,
            )?;

            add_clone_where_predicates(&mut o_st.generics, &clone_type_params);
            add_to_owned_where_predicates(cx, &mut o_st.generics, &borrowed_type_params);
            borrowed_type_params_out.extend(borrowed_type_params);
            clone_type_params_out.extend(clone_type_params);

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

            let borrow_fn = if needs_mut {
                quote! {
                    #[inline]
                    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
                        #borrow_ident {
                            #(#borrow_entries,)*
                        }
                    }
                }
            } else {
                quote! {
                    #[inline]
                    fn borrow(&self) -> Self::Target<'_> {
                        #borrow_ident {
                            #(#borrow_entries,)*
                        }
                    }
                }
            };

            (to_owned_fn, borrow_fn)
        }
        (syn::Item::Enum(o_en), syn::Item::Enum(b_en)) => {
            let attr = attr::container(cx, attrs, &o_en.attrs)?;
            attr::strip([&mut o_en.attrs, &mut b_en.attrs]);

            apply_attributes(&attr.attributes, &mut o_en.attrs, &mut b_en.attrs);
            process_generics(
                cx,
                o_en.ident.span(),
                &mut o_en.generics,
                o_en.variants.iter().all(|v| v.fields.is_empty()),
            );
            o_en.ident = attr.owned_ident(&o_en.ident);

            let mut to_owned_variants = Vec::new();
            let mut borrow_variants = Vec::new();
            let type_params = collect_type_param_idents(&b_en.generics);
            let mut clone_type_params = HashSet::new();
            let mut borrowed_type_params = HashSet::new();

            let owned_ident = o_en.ident.clone();
            let borrow_ident = b_en.ident.clone();

            for (o_variant, b_variant) in o_en.variants.iter_mut().zip(b_en.variants.iter_mut()) {
                let attr = attr::variant(cx, &o_variant.attrs, &attr)?;
                attr::strip([&mut o_variant.attrs, &mut b_variant.attrs]);

                apply_attributes(&attr.attributes, &mut o_variant.attrs, &mut b_variant.attrs);

                let mut to_owned_entries = Vec::new();
                let mut borrow_entries = Vec::new();

                process_fields(
                    cx,
                    Access::BindingAccess,
                    attr.kind,
                    &mut o_variant.fields,
                    &mut b_variant.fields,
                    &mut to_owned_entries,
                    &mut borrow_entries,
                    &mut needs_mut,
                    &type_params,
                    &mut clone_type_params,
                    &mut borrowed_type_params,
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

            let borrow_fn = if needs_mut {
                quote! {
                    #[inline]
                    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
                        match self {
                            #(#borrow_variants,)*
                        }
                    }
                }
            } else {
                quote! {
                    #[inline]
                    fn borrow(&self) -> Self::Target<'_> {
                        match self {
                            #(#borrow_variants,)*
                        }
                    }
                }
            };

            add_clone_where_predicates(&mut o_en.generics, &clone_type_params);
            add_to_owned_where_predicates(cx, &mut o_en.generics, &borrowed_type_params);
            borrowed_type_params_out.extend(borrowed_type_params);
            clone_type_params_out.extend(clone_type_params);

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
        let mut borrow_generics_with_bounds = borrow_generics.clone();
        add_clone_where_predicates(&mut borrow_generics_with_bounds, &clone_type_params_out);
        add_to_owned_where_predicates(cx, &mut borrow_generics_with_bounds, &borrowed_type_params_out);
        let (impl_generics, type_generics, where_generics) = borrow_generics_with_bounds.split_for_impl();
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

        let mut owned_generics_for_borrow = owned_generics.clone();
        add_borrow_where_predicates(cx, &mut owned_generics_for_borrow, &borrowed_type_params_out, needs_mut);
        let (impl_generics, type_generics, where_generics) = owned_generics_for_borrow.split_for_impl();

        // Build `T: 'this` where predicates for all type params that appear in borrowed fields
        // (both those using the borrowme ToOwned path and those using the Std clone/ref path).
        let type_param_lifetime_bounds: Vec<syn::WherePredicate> = borrowed_type_params_out
            .iter()
            .chain(clone_type_params_out.iter())
            .map(|ident| syn::parse_quote!(#ident: #this_lt))
            .collect();

        let gat_where = if type_param_lifetime_bounds.is_empty() {
            quote! {}
        } else {
            quote! { where #(#type_param_lifetime_bounds,)* }
        };

        if needs_mut {
            let borrow_mut_t = &cx.borrowme_borrow_mut_t;

            quote! {
                #[automatically_derived]
                impl #impl_generics #borrow_mut_t for #owned_ident #type_generics #where_generics {
                    type TargetMut<#this_lt> #gat_where = #borrow_ident #borrow_return_type_generics;
                    #borrow_fn
                }
            }
        } else {
            let borrow_t = &cx.borrowme_borrow_t;

            quote! {
                #[automatically_derived]
                impl #impl_generics #borrow_t for #owned_ident #type_generics #where_generics {
                    type Target<#this_lt> #gat_where = #borrow_ident #borrow_return_type_generics;
                    #borrow_fn
                }
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
    default_kind: Option<(Span, attr::FieldTypeKind)>,
    o_fields: &mut syn::Fields,
    b_fields: &mut syn::Fields,
    to_owned_entries: &mut Vec<syn::FieldValue>,
    borrow_entries: &mut Vec<syn::FieldValue>,
    parent_needs_mut: &mut bool,
    type_params: &HashSet<syn::Ident>,
    // Type params that need `T: Clone` bound (used in `Std` path for direct refs to type params).
    clone_type_params: &mut HashSet<syn::Ident>,
    // Type params that need `T: 'this` in the GAT and `T: ::borrowme::ToOwned` in impls.
    borrowed_type_params: &mut HashSet<syn::Ident>,
) -> Result<(), ()> {
    for (index, (o_field, b_field)) in o_fields.iter_mut().zip(b_fields.iter_mut()).enumerate() {
        let field_ty_spans = field_ty_spans(o_field);

        let mut attr = attr::field(cx, field_ty_spans, &o_field.attrs, default_kind)?;
        attr::strip([&mut o_field.attrs, &mut b_field.attrs]);
        apply_attributes(&attr.attributes, &mut o_field.attrs, &mut b_field.attrs);

        // Ensure that the field does not make use of any lifetimes.
        let ignore = HashSet::new();
        let mut lifetimes = Vec::new();
        let mut as_ty = o_field.ty.clone();

        let (type_hint, immediate_reference) = process_type(&mut as_ty, &ignore, &mut lifetimes);

        let needs_mut = lifetimes
            .iter()
            .any(|(_, _, mut_token)| mut_token.is_some());
        let needs_mut = attr.is_mut() || needs_mut;
        *parent_needs_mut |= needs_mut;

        // Provide diagnostics in case there are field lifetimes we can't
        // make anything out of. Such as a `&'a str` field marked with
        // `#[copy]`.
        match attr.ty.kind() {
            attr::FieldTypeKind::Copy(true) => {
                for (span, lt, _) in lifetimes {
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
                // If the immediate reference target is a bare type parameter (e.g. `&'a T`
                // where `T` is from the struct's own generics), treat it like `#[borrowme(std)]`:
                // the owned field stores `T` directly, and we use clone/ref for conversion.
                let is_type_param_ref = immediate_reference.as_ref().is_some_and(|inner| {
                    is_type_param(inner, type_params)
                });

                if is_type_param_ref
                    && matches!(attr.ty.kind(), attr::FieldTypeKind::Default)
                    && attr.ty.owned.is_none()
                {
                    attr.ty.set_kind(attr::FieldTypeKind::Std);
                    // Track which type params use the Std (clone+ref) path so we can add
                    // `T: Clone` and `T: 'this` bounds.
                    if let Some(syn::Type::Path(tp)) = &immediate_reference {
                        if let Some(ident) = tp.path.get_ident() {
                            clone_type_params.insert(ident.clone());
                        }
                    }
                }

                let is_std_ref = matches!(attr.ty.kind(), attr::FieldTypeKind::Std if immediate_reference.is_some());

                // For non-copy types, build an expression that tries to use the
                // `ToOwned` implementation to figure out which type to use.
                match type_hint {
                    TypeHint::None
                        if attr.ty.owned.is_none() && !is_std_ref && !lifetimes.is_empty() =>
                    {
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

                        attr.ty.set_owned(Respan::new(ty, field_ty_spans));
                    }
                    TypeHint::Copy => {
                        if !matches!(attr.ty.kind(), attr::FieldTypeKind::Copy(false)) {
                            attr.ty.set_kind(attr::FieldTypeKind::Copy(true));
                        }
                    }
                    _ => {}
                }
            }
        };

        let (to_owned, borrow) = match (attr.ty.kind(), &immediate_reference, attr.ty.owned()) {
            (attr::FieldTypeKind::Copy(true), _, _) => (Call::Ref, Call::Ref),
            (attr::FieldTypeKind::Std, _, Some(ty)) => {
                o_field.ty = ty.as_type();
                (Call::Path(&cx.clone_t_clone), Call::Ref)
            }
            (attr::FieldTypeKind::Std, Some(ty), None) => {
                o_field.ty = ty.clone();
                (Call::Path(&cx.clone_t_clone), Call::Ref)
            }
            (_, _, Some(ty)) => {
                o_field.ty = ty.as_type();
                collect_type_params_in_type(&b_field.ty, type_params, borrowed_type_params);

                let borrow = if needs_mut {
                    attr.borrow_mut(cx)
                } else {
                    attr.borrow(cx)
                };

                (Call::Path(attr.to_owned(cx)), Call::Path(borrow))
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

        let is_copy = matches!(attr.ty.kind(), attr::FieldTypeKind::Copy(true));

        let bound = BoundAccess {
            use_reference: !is_copy && immediate_reference.is_none(),
            is_mut: false,
            access,
            binding: &binding,
        };

        to_owned_entries.push(syn::FieldValue {
            attrs: Vec::new(),
            member: member.clone(),
            colon_token: Some(<Token![:]>::default()),
            expr: to_owned.as_expr(&bound),
        });

        let bound = BoundAccess {
            use_reference: !is_copy,
            is_mut: needs_mut,
            access,
            binding: &binding,
        };

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
fn process_type(
    ty: &mut syn::Type,
    ignore: &HashSet<syn::Ident>,
    out: &mut Vec<(Span, Option<syn::Lifetime>, Option<Token![mut]>)>,
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
                ty.mutability.take(),
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

fn process_generic_type<P>(
    generics: &mut Punctuated<syn::GenericArgument, P>,
    ignore: &HashSet<syn::Ident>,
    out: &mut Vec<(Span, Option<syn::Lifetime>, Option<Token![mut]>)>,
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
                    None,
                ));
            }
            syn::GenericArgument::Type(ty) => {
                process_type(ty, ignore, out);
            }
            _ => {}
        }
    }
}

/// Check whether a type is a bare type parameter (a single-segment path that
/// matches one of the known type param idents).
fn is_type_param(ty: &syn::Type, type_params: &HashSet<syn::Ident>) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(ident) = tp.path.get_ident() {
            return type_params.contains(ident);
        }
    }
    false
}

/// Collect type parameter idents (from `type_params`) that appear anywhere in `ty`,
/// inserting them into `out`.
fn collect_type_params_in_type(
    ty: &syn::Type,
    type_params: &HashSet<syn::Ident>,
    out: &mut HashSet<syn::Ident>,
) {
    match ty {
        syn::Type::Path(ty) => {
            if let Some(ident) = ty.path.get_ident() {
                if type_params.contains(ident) {
                    out.insert(ident.clone());
                }
            }
            for seg in &ty.path.segments {
                match &seg.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        for arg in &args.args {
                            match arg {
                                syn::GenericArgument::Type(t) => {
                                    collect_type_params_in_type(t, type_params, out);
                                }
                                _ => {}
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(args) => {
                        for t in &args.inputs {
                            collect_type_params_in_type(t, type_params, out);
                        }
                    }
                    _ => {}
                }
            }
        }
        syn::Type::Reference(ty) => {
            collect_type_params_in_type(&ty.elem, type_params, out);
        }
        syn::Type::Slice(ty) => {
            collect_type_params_in_type(&ty.elem, type_params, out);
        }
        syn::Type::Array(ty) => {
            collect_type_params_in_type(&ty.elem, type_params, out);
        }
        syn::Type::Tuple(ty) => {
            for elem in &ty.elems {
                collect_type_params_in_type(elem, type_params, out);
            }
        }
        syn::Type::Group(ty) => {
            collect_type_params_in_type(&ty.elem, type_params, out);
        }
        _ => {}
    }
}

/// Collect the idents of all type parameters from a generics declaration.
fn collect_type_param_idents(generics: &syn::Generics) -> HashSet<syn::Ident> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(t.ident.clone()),
            _ => None,
        })
        .collect()
}

/// Add `T: Clone` where predicates for each ident in `type_params` to the given generics.
fn add_clone_where_predicates(generics: &mut syn::Generics, type_params: &HashSet<syn::Ident>) {
    if type_params.is_empty() {
        return;
    }

    let where_clause = generics.make_where_clause();

    for ident in type_params {
        where_clause
            .predicates
            .push(syn::parse_quote!(#ident: ::core::clone::Clone));
    }
}

/// Add `T: ::borrowme::ToOwned` where predicates for each ident in `type_params`
/// to the given generics.
fn add_to_owned_where_predicates(
    cx: &Ctxt,
    generics: &mut syn::Generics,
    type_params: &HashSet<syn::Ident>,
) {
    if type_params.is_empty() {
        return;
    }

    let to_owned_t = &cx.borrowme_to_owned_t;
    let where_clause = generics.make_where_clause();

    for ident in type_params {
        where_clause
            .predicates
            .push(syn::parse_quote!(#ident: #to_owned_t));
    }
}

/// Add `<T as ::borrowme::ToOwned>::Owned: ::borrowme::Borrow` (or BorrowMut)
/// where predicates for the `Borrow`/`BorrowMut` impl.
fn add_borrow_where_predicates(
    cx: &Ctxt,
    generics: &mut syn::Generics,
    type_params: &HashSet<syn::Ident>,
    needs_mut: bool,
) {
    if type_params.is_empty() {
        return;
    }

    let to_owned_t = &cx.borrowme_to_owned_t;
    let borrow_t = if needs_mut {
        &cx.borrowme_borrow_mut_t
    } else {
        &cx.borrowme_borrow_t
    };
    let where_clause = generics.make_where_clause();

    for ident in type_params {
        where_clause
            .predicates
            .push(syn::parse_quote!(<#ident as #to_owned_t>::Owned: #borrow_t));
    }
}

/// Strip lifetime parameters from the given generics.
fn process_generics(cx: &Ctxt, span: Span, generics: &mut syn::Generics, empty_type: bool) {
    let mut any = false;

    let mut params = generics.params.clone();
    params.clear();

    for p in &generics.params {
        if matches!(p, syn::GenericParam::Lifetime(..)) {
            any = true;
        } else {
            params.push(p.clone());
        }
    }

    if !any && !empty_type {
        let span = if !generics.params.is_empty() {
            generics.params.span()
        } else {
            span
        };

        cx.span_error(
            span,
            format_args!("{NAME}: Can only be used on types which receive lifetimes or are empty"),
        );
    }

    generics.params = params;
}
