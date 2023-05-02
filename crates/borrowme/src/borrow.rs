#[cfg(feature = "std")]
mod std;

/// Borrow from self.
///
/// This works similarly to [`Borrow`][std::borrow::Borrow] but allows borrowing
/// compoundly from `&self` by defining a [generic `Target`]. This means it's
/// not just limited to returning an immediate reference to the borrowed value
/// but can return something which receives lifetime parameters that borrows
/// from self. This is called "compound borrowing" because it lets you return
/// types which contains compound references.
///
/// It is recommended that you use [`borrow`][crate::borrow()] instead of
/// importing this trait.
///
/// <br>
///
/// # What about `std::borrow::Borrow`?
///
/// The [`Borrow`][std::borrow::Borrow] trait as defined can't perform compound
/// borrows from `&self`. Because the `borrow` method immediately returns *a
/// reference* to the borrowed type.
///
/// ```
/// trait Borrow<Borrowed: ?Sized> {
///     fn borrow(&self) -> &Borrowed;
/// }
/// ```
///
/// This means that there is no way to implement something like
/// `Borrow<Word<'a>>` because it's required that we return a reference which
/// doesn't outlive `'a`, something that can't be satisfied from the call to
/// `&self`.
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
///  --> src/borrow.rs:37:9
///   |
/// 9 |         &Word(self.0.as_str())
///   |         ^---------------------
///   |         ||
///   |         |temporary value created here
///   |         returns a reference to data owned by the current function
/// ```
///
/// The solution implemented in this crate is to use a [generic `Target`], with
/// which we can implement `borrow` like this:
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
/// A catch here is that `Borrow` can only be implemented once for each time,
/// compared to [`Borrow<T>`][std::borrow::Borrow]. But for our purposes this is
/// fine. This crate is primarily intended to work with two symmetrical types
/// and any deviation from that pattern can be handled by customizing the
/// behavior of the [`#[borrowme]`][crate::borrowme] attribute.
///
/// [generic `Target`]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
pub trait Borrow {
    type Target<'a>
    where
        Self: 'a;

    /// Borrow from `self`.
    fn borrow(&self) -> Self::Target<'_>;
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

impl<T> Borrow for [T] {
    type Target<'a> = &'a [T] where T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        self
    }
}
