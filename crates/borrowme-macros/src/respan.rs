use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::ToTokens;

pub(crate) struct Respan<T> {
    inner: T,
    spans: (Span, Span),
}

impl<T> Respan<T> {
    pub(crate) fn new(inner: T, spans: (Span, Span)) -> Self {
        Self { inner, spans }
    }
}

impl Respan<syn::Type> {
    // Make use of the verbatim type to generate a respanned token stream.
    pub(crate) fn into_type(&self) -> syn::Type {
        let stream = self.inner.to_token_stream();
        syn::Type::Verbatim(crate::respan::respan(stream, self.spans))
    }
}

pub(crate) fn respan(stream: TokenStream, spans: (Span, Span)) -> TokenStream {
    let mut it = stream.into_iter();

    let first = it.next();
    first
        .into_iter()
        .map(|t| inner(t, spans.0))
        .chain(it.map(|t| inner(t, spans.1)))
        .collect()
}

fn respan_stream(stream: TokenStream, span: Span) -> TokenStream {
    stream.into_iter().map(|t| inner(t, span)).collect()
}

fn inner(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan_stream(g.stream(), span));
    }
    token.set_span(span);
    token
}
