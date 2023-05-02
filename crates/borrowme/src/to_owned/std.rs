use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::hash::Hash;
use std::path::{Path, PathBuf};

use crate::ToOwned;

impl ToOwned for str {
    type Owned = String;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        String::from(self)
    }
}

impl ToOwned for String {
    type Owned = String;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        String::from(self.as_str())
    }
}

impl ToOwned for &mut String {
    type Owned = String;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        String::from(self.as_str())
    }
}

impl ToOwned for &mut str {
    type Owned = String;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        String::from(&**self)
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

impl<B> ToOwned for Cow<'_, B>
where
    B: 'static + ?Sized + std::borrow::ToOwned,
{
    type Owned = Cow<'static, B>;

    #[inline]
    fn to_owned(&self) -> <Self as ToOwned>::Owned {
        // Cloning the cow will either clone the inner value - if it's already
        // present - or the associated reference.
        Cow::Owned(self.clone().into_owned())
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_path_buf()
    }
}

impl ToOwned for OsStr {
    type Owned = OsString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_os_string()
    }
}

impl ToOwned for CStr {
    type Owned = CString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        std::borrow::ToOwned::to_owned(self)
    }
}

macro_rules! seq {
    (cap $seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
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
