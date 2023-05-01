use std::collections::HashSet;
use std::hash::Hash;

/// A [`ToOwned`]-like trait which doesn't require [`std::borrow::Borrow`] to be
/// implemtned.
pub trait ToOwned {
    /// The owned type this is being converted to.
    type Owned;

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
