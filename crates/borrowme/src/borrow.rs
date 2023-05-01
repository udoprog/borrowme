use std::collections::HashSet;
use std::hash::Hash;

/// A Deref-like trait that can borrow from `&self`.
pub trait Borrow {
    type Target<'a>
    where
        Self: 'a;

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
        match self {
            Some(some) => Some(some.borrow()),
            None => None,
        }
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
