use std::collections::HashSet;
use std::hash::Hash;

/// Convert a type to owned.
///
/// This works similarly to [`ToOwned`][std::borrow::ToOwned] with a few relaxed
/// constaints. It is recommended that you use [`to_owned`][crate::to_owned()]
/// instead of importing this trait.
///
/// <br>
///
/// # What about `std::borrow::ToOwned`?
///
/// [`ToOwned`][`std::borrow::ToOwned`] is a symmetric trait, which explicitly
/// requires that the resulting `Owned` associated type can be borrowed back
/// into a reference of itself (See [`Borrow<Self>`][crate::Borrow] in this
/// crate for more details). So because we can't implement
/// [`Borrow<Self>`][std::borrow::Borrow], we can't implemented
/// [`ToOwned`][std::borrow::ToOwned] either.
///
/// Let's say we want to implement [`ToOwned`][std::borrow::ToOwned] for a type
/// which has a lifetime:
///
/// ```compile_fail
/// struct Word<'a>(&'a str);
/// struct OwnedWord(String);
///
/// impl ToOwned for Word<'_> {
///     type Owned = OwnedWord;
///
///     fn to_owned(&self) -> OwnedWord {
///         OwnedWord(self.0.to_owned())
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
/// require the produced value to be [`Borrow<Self>`][std::borrow::Borrow].
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

    /// Perform a covnersion from a reference to owned data.
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

impl ToOwned for str {
    type Owned = String;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        std::borrow::ToOwned::to_owned(self)
    }
}

impl<T> ToOwned for Option<T>
where
    T: ToOwned,
{
    type Owned = Option<T::Owned>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.as_ref().map(ToOwned::to_owned)
    }
}

impl<T> ToOwned for Vec<T>
where
    T: ToOwned,
{
    type Owned = Vec<T::Owned>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        let mut out = Vec::with_capacity(self.len());

        for value in self.iter() {
            out.push(value.to_owned());
        }

        out
    }
}

impl<T> ToOwned for HashSet<T>
where
    T: ToOwned,
    T::Owned: Hash + Eq,
{
    type Owned = HashSet<T::Owned>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        let mut out = HashSet::with_capacity(self.len());

        for value in self.iter() {
            out.insert(value.to_owned());
        }

        out
    }
}
