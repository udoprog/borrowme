#[cfg(feature = "std")]
mod std;

/// Convert to owned.
///
/// This works similarly to [`ToOwned`][::std::borrow::ToOwned] with a few
/// relaxed constaints. It is recommended that you use
/// [`to_owned`][crate::to_owned()] instead of importing this trait.
///
/// <br>
///
/// # What about `std::borrow::ToOwned`?
///
/// [`std::borrow::ToOwned`] a trait which requires that the resulting `Owned`
/// type can be borrowed back into a reference of itself. This can't be
/// implemented for compound borrowing (See
/// [`borrowme::Borrow`][crate::Borrow]). So because we can't implement
/// [`std::borrow::Borrow<Self>`][::std::borrow::Borrow], we can't implemented
/// [`std::borrow::ToOwned`] either.
///
/// To showcase this, let's try to implement [`std::borrow::ToOwned`] for a type
/// which has a lifetime parameter:
///
/// [`std::borrow::ToOwned`]: ::std::borrow::ToOwned
///
/// ```rust,no_run
/// # use borrowme::ToOwned;
/// struct Word<'a>(&'a str);
/// struct OwnedWord(String);
///
/// impl ToOwned for Word<'_> {
///     type Owned = OwnedWord;
///
///     fn to_owned(&self) -> OwnedWord {
///         OwnedWord(String::from(self.0))
///     }
/// }
/// ```
///
/// ```text
/// error[E0277]: the trait bound `OwnedWord: std::borrow::Borrow<Word<'_>>` is not satisfied
///   --> src/lib.rs:27:18
///    |
/// 11 |     type Owned = OwnedWord;
///    |                  ^^^^^^^^^ the trait `std::borrow::Borrow<Word<'_>>` is not implemented for `OwnedWord`
/// ```
///
/// So in this crate we define a different [`ToOwned`] trait which does not
/// require the produced value to be [`Borrow<Self>`][::std::borrow::Borrow].
///
/// With this, we can implement the conversion:
///
/// ```
/// # struct Word<'a>(&'a str);
/// # struct OwnedWord(String);
/// use borrowme::ToOwned;
///
/// impl ToOwned for Word<'_> {
///     type Owned = OwnedWord;
///
///     fn to_owned(&self) -> OwnedWord {
///         OwnedWord(self.0.to_string())
///     }
/// }
/// ```
pub trait ToOwned {
    /// The owned type this is being converted to.
    type Owned;

    /// Perform a covnersion from a reference to owned value.
    fn to_owned(&self) -> Self::Owned;
}

impl<T> ToOwned for &T
where
    T: ?Sized + ToOwned,
{
    type Owned = T::Owned;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        T::to_owned(*self)
    }
}
