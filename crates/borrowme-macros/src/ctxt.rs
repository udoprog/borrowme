use std::cell::RefCell;
use std::fmt;

use proc_macro2::{Span, TokenStream};

pub(crate) struct Ctxt {
    errors: RefCell<Vec<syn::Error>>,
    pub(crate) borrowme_borrow_t_borrow: syn::Path,
    pub(crate) borrowme_borrow_mut_t_borrow_mut: syn::Path,
    pub(crate) clone_t_clone: syn::Path,
    pub(crate) borrowme_borrow_t: syn::Path,
    pub(crate) borrowme_borrow_mut_t: syn::Path,
    pub(crate) borrowme_to_owned_t: syn::Path,
    pub(crate) borrowme_to_owned_t_to_owned: syn::Path,
}

impl Ctxt {
    pub(crate) fn new(span: Span) -> Self {
        Self {
            errors: RefCell::new(Vec::new()),
            borrowme_borrow_t_borrow: path(span, ["borrowme", "Borrow", "borrow"]),
            borrowme_borrow_mut_t_borrow_mut: path(span, ["borrowme", "BorrowMut", "borrow_mut"]),
            borrowme_borrow_t: path(span, ["borrowme", "Borrow"]),
            borrowme_borrow_mut_t: path(span, ["borrowme", "BorrowMut"]),
            borrowme_to_owned_t: path(span, ["borrowme", "ToOwned"]),
            clone_t_clone: path(span, ["core", "clone", "Clone", "clone"]),
            borrowme_to_owned_t_to_owned: path(span, ["borrowme", "ToOwned", "to_owned"]),
        }
    }

    /// Convert context into any registered errors.
    pub(crate) fn into_errors(self) -> TokenStream {
        let errors = self.errors.into_inner();

        let mut stream = TokenStream::new();

        for error in errors {
            stream.extend(error.to_compile_error());
        }

        stream
    }

    /// Record an error.
    pub(crate) fn error(&self, error: syn::Error) {
        self.errors.borrow_mut().push(error);
    }

    /// Record a spanned error.
    pub(crate) fn span_error<T>(&self, span: Span, message: T)
    where
        T: fmt::Display,
    {
        self.error(syn::Error::new(span, message));
    }

    /// Check if context has errors.
    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }
}

/// Helper to construct a path.
pub(crate) fn path<I>(span: Span, parts: I) -> syn::Path
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut path = syn::Path {
        leading_colon: Some(<syn::Token![::]>::default()),
        segments: syn::punctuated::Punctuated::default(),
    };

    for part in parts {
        path.segments.push(syn::PathSegment {
            ident: syn::Ident::new(part.as_ref(), span),
            arguments: syn::PathArguments::None,
        });
    }

    path
}
