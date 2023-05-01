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
/// # What about `std::borrow::Borrow`?
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
/// struct Word<'a>(&'a str);
/// struct OwnedWord(String);
///
/// impl<'a> Borrow<Word<'a>> for OwnedWord {
///     fn borrow(&self) -> &Word<'a> {
///         &Word(self.0.as_str())
///     }
/// }
/// ```
///
/// ```text
/// error[E0515]: cannot return reference to temporary value
///  --> src\borrow.rs:37:9
///   |
/// 9 |         &Word(self.0.as_str())
///   |         ^---------------------
///   |         ||
///   |         |temporary value created here
///   |         returns a reference to data owned by the current function
/// ```
///
/// The solution implemented in this crate is to use a [generic `Target`], with
/// this we can implement `borrow` like this:
///
/// ```
/// # struct Word<'a>(&'a str);
/// # struct OwnedWord(String);
/// use borrowme::Borrow;
///
/// impl Borrow for OwnedWord {
///     type Target<'a> = Word<'a>;
///
///     fn borrow(&self) -> Self::Target<'_> {
///         Word(self.0.as_str())
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
