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

impl<T> ToTokens for Respan<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = respan(self.inner.to_token_stream(), self.spans);
        inner.to_tokens(tokens);
    }
}

pub(crate) fn respan(stream: TokenStream, spans: (Span, Span)) -> TokenStream {
    let mut it = stream.into_iter().peekable();

    let mut out = TokenStream::new();

    while it.peek().is_some() {
        out.extend(it.next().map(|t| inner(t, spans.0)));
    }

    out.extend(it.next().map(|t| inner(t, spans.1)));
    out
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
