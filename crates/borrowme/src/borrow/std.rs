use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::hash::Hash;
use std::path::{Path, PathBuf};

use crate::Borrow;

impl Borrow for String {
    type Target<'a> = &'a str;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        self.as_str()
    }
}

impl<B> Borrow for Cow<'static, B>
where
    B: ?Sized + std::borrow::ToOwned,
{
    type Target<'a> = Cow<'a, B>;

    #[inline]
    fn borrow(&self) -> Self::Target<'_> {
        // This works because Cow implements `Deref<Target = B>`.
        Cow::Borrowed(self)
    }
}

// Trivial implementation for deref types.
macro_rules! deref {
    ($from:ty, $to:ty) => {
        impl Borrow for $from {
            type Target<'a> = &'a $to where Self: 'a;

            #[inline]
            fn borrow(&self) -> Self::Target<'_> {
                self
            }
        }
    };
}

deref!(PathBuf, Path);
deref!(OsString, OsStr);
deref!(CString, CStr);

macro_rules! seq {
    (cap $seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<T> Borrow for $seq<T>
        where
            T: Borrow,
            $(for<'a> T::Target<'a>: $trait,)*
        {
            type Target<'a> = $seq<T::Target<'a>> where T: 'a;

            #[inline]
            fn borrow(&self) -> Self::Target<'_> {
                let mut out = <$seq<_>>::with_capacity(self.len());

                for value in self {
                    out.$insert(value.borrow());
                }

                out
            }
        }
    };

    ($seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<T> Borrow for $seq<T>
        where
            T: Borrow,
            $(for<'a> T::Target<'a>: $trait,)*
        {
            type Target<'a> = $seq<T::Target<'a>> where T: 'a;

            #[inline]
            fn borrow(&self) -> Self::Target<'_> {
                let mut out = <$seq<_>>::new();

                for value in self {
                    out.$insert(value.borrow());
                }

                out
            }
        }
    };
}

macro_rules! map {
    (cap $map:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<K, V> Borrow for $map<K, V>
        where
            K: Borrow,
            V: Borrow,
            $(for<'a> K::Target<'a>: $trait,)*
        {
            type Target<'a> = $map<K::Target<'a>, V::Target<'a>> where K: 'a, V: 'a;

            #[inline]
            fn borrow(&self) -> Self::Target<'_> {
                let mut out = <$map<_, _>>::with_capacity(self.len());

                for (key, value) in self {
                    out.$insert(key.borrow(), value.borrow());
                }

                out
            }
        }
    };

    ($map:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<K, V> Borrow for $map<K, V>
        where
            K: Borrow,
            V: Borrow,
            $(for<'a> K::Target<'a>: $trait,)*
        {
            type Target<'a> = $map<K::Target<'a>, V::Target<'a>> where K: 'a, V: 'a;

            #[inline]
            fn borrow(&self) -> Self::Target<'_> {
                let mut out = <$map<_, _>>::new();

                for (key, value) in self {
                    out.$insert(key.borrow(), value.borrow());
                }

                out
            }
        }
    };
}

seq!(cap Vec, push);
seq!(cap HashSet, insert, Hash, Eq);
seq!(BTreeSet, insert, PartialOrd, Ord, Eq);
seq!(LinkedList, push_back);

map!(cap HashMap, insert, Hash, Eq);
map!(BTreeMap, insert, PartialOrd, Ord, Eq);
