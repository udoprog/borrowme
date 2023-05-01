use std::collections::HashSet;
use std::hash::Hash;

/// Borrow from self.
///
/// This works similarly to [`Borrow`][std::borrow::Borrow] but allows borrowing
/// from `&self` by defining a [generic `Target`].
///
/// It is recommended that you use [`borrow`][crate::borrow()] instead of
/// importing this trait.
///
/// <br>
///
/// # Why can't we use `std::borrow::Borrow`?
///
/// The [`Borrow`][std::borrow::Borrow] trait as defined faces an issue which
/// can't be easily addressed if we want to perform complex borrow from `&self`.
/// The `borrow` method immediately returns *a reference* to the borrowed type.
///
/// ```
/// pub trait Borrow<Borrowed: ?Sized> {
///     fn borrow(&self) -> &Borrowed;
/// }
/// ```
///
/// This means that there is no way to implement `Borrow<Word<'a>>` because it
/// required that we return a reference which doesn't outlive `'a`, something
/// that can't be satisfied because we don't hold a reference to `Word<'a>`.
///
/// ```compile_fail
/// # use std::borrow::Borrow;
/// struct Word<'a> {
///     text: &'a str,
///     lang: Option<&'a str>
/// }
///
/// struct OwnedWord {
///     text: String,
///     lang: Option<String>
/// }
///
/// impl<'a> Borrow<Word<'a>> for OwnedWord {
///     fn borrow(&self) -> &Word<'a> {
///         &Word {
///            text: self.text.as_str(),
///            lang: self.lang.as_ref().map(String::as_str),
///         }
///     }
/// }
/// ```
///
/// ```text
/// error: lifetime may not live long enough
///   --> src/lib.rs:83:9
///    |
/// 6  |   impl<'a> Borrow<Word<'a>> for OwnedWord {
///    |        -- lifetime `'a` defined here
/// 7  |       fn borrow(&self) -> &Word<'a> {
///    |                 - let's call the lifetime of this reference `'1`
/// 8  | /         &Word {
/// 9  | |            text: self.text.as_str(),
/// 10 | |            lang: self.lang.as_ref().map(String::as_str),
/// 11 | |         }
///    | |_________^ associated function was supposed to return data with lifetime `'a` but it is returning data with lifetime `'1`
/// ```
///
/// The solution implemented in this crate is to use a [generic `Target`], with
/// this we can implement `borrow` like this:
///
/// ```
/// # struct Word<'a> { text: &'a str, lang: Option<&'a str> }
/// # struct OwnedWord { text: String, lang: Option<String> }
/// use borrowme::Borrow;
///
/// impl Borrow for OwnedWord {
///     type Target<'a> = Word<'a>;
///
///     fn borrow(&self) -> Self::Target<'_> {
///         Word {
///            text: self.text.as_str(),
///            lang: self.lang.as_ref().map(String::as_str),
///         }
///     }
/// }
/// ```
///
/// > **Note::** A catch here is that `Borrow` can only be implemented once for
/// > each time, compared to [`Borrow<T>`][std::borrow::Borrow]. But for our
/// > purposes this is fine. This crate is primarily intended to work with two
/// > symmetrical types.
///
/// [generic `Target`]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
pub trait Borrow {
    type Target<'a>
    where
        Self: 'a;

    /// Borrow from `self`.
    fn borrow(&self) -> Self::Target<'_>;
}

impl Borrow for String {
    type Target<'a> = &'a str;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        self.as_str()
    }
}

impl<T> Borrow for Option<T>
where
    T: Borrow,
{
    type Target<'a> = Option<T::Target<'a>> where T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        self.as_ref().map(|some| some.borrow())
    }
}

impl<T> Borrow for Vec<T>
where
    T: Borrow,
{
    type Target<'a> = Vec<T::Target<'a>> where T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        let mut out = Vec::with_capacity(self.len());

        for value in self {
            out.push(value.borrow());
        }

        out
    }
}

impl<T> Borrow for HashSet<T>
where
    T: Borrow,
    for<'a> T::Target<'a>: Eq + Hash,
{
    type Target<'a> = HashSet<T::Target<'a>> where T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        let mut out = HashSet::with_capacity(self.len());

        for value in self {
            out.insert(value.borrow());
        }

        out
    }
}
