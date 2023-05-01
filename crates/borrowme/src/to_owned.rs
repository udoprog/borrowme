#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};

/// Convert to owned.
///
/// This works similarly to [`ToOwned`][std::borrow::ToOwned] with a few relaxed
/// constaints. It is recommended that you use [`to_owned`][crate::to_owned()]
/// instead of importing this trait.
///
/// <br>
///
/// # What about `std::borrow::ToOwned`?
///
/// [`std::borrow::ToOwned`] a trait which requires that the resulting `Owned`
/// type can be borrowed back into a reference of itself. This can't be
/// implemented for compound borrowing (See
/// [`borrowme::Borrow`][crate::Borrow]). So because we can't implement
/// [`std::borrow::Borrow<Self>`], we can't implemented [`std::borrow::ToOwned`]
/// either.
///
/// To showcase this, let's try to implement [`std::borrow::ToOwned`] for a type
/// which has a lifetime parameter:
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

#[cfg(feature = "std")]
impl ToOwned for str {
    type Owned = String;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        String::from(self)
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

#[cfg(feature = "std")]
impl<T> ToOwned for [T]
where
    T: Clone,
{
    type Owned = Vec<T>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_vec()
    }
}

macro_rules! seq {
    (cap $seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        #[cfg(feature = "std")]
        impl<T> ToOwned for $seq<T>
        where
            T: ToOwned,
            $(T::Owned: $trait,)*
        {
            type Owned = $seq<T::Owned>;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                let mut out = <$seq<T::Owned>>::with_capacity(self.len());

                for value in self.iter() {
                    out.$insert(value.to_owned());
                }

                out
            }
        }
    };

    ($seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        #[cfg(feature = "std")]
        impl<T> ToOwned for $seq<T>
        where
            T: ToOwned,
            $(T::Owned: $trait,)*
        {
            type Owned = $seq<T::Owned>;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                let mut out = <$seq<T::Owned>>::new();

                for value in self.iter() {
                    out.$insert(value.to_owned());
                }

                out
            }
        }
    };
}

macro_rules! map {
    (cap $map:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        #[cfg(feature = "std")]
        impl<K, V> ToOwned for $map<K, V>
        where
            K: ToOwned,
            V: ToOwned,
            $(K::Owned: $trait,)*
        {
            type Owned = $map<K::Owned, V::Owned>;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                let mut out = <$map<_, _>>::with_capacity(self.len());

                for (key, value) in self.iter() {
                    out.$insert(key.to_owned(), value.to_owned());
                }

                out
            }
        }
    };

    ($map:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        #[cfg(feature = "std")]
        impl<K, V> ToOwned for $map<K, V>
        where
            K: ToOwned,
            V: ToOwned,
            $(K::Owned: $trait,)*
        {
            type Owned = $map<K::Owned, V::Owned>;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                let mut out = <$map<_, _>>::new();

                for (key, value) in self.iter() {
                    out.$insert(key.to_owned(), value.to_owned());
                }

                out
            }
        }
    };
}

seq!(cap HashSet, insert, Hash, Eq);
seq!(cap Vec, push);
seq!(BTreeSet, insert, PartialOrd, Ord, Eq);
seq!(LinkedList, push_back);

map!(cap HashMap, insert, Hash, Eq);
map!(BTreeMap, insert, PartialOrd, Ord, Eq);
