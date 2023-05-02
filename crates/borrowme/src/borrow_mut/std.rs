use std::collections::{BTreeMap, HashMap, LinkedList};
use std::hash::Hash;

use crate::{Borrow, BorrowMut};

impl BorrowMut for String {
    type TargetMut<'a> = &'a mut String;

    #[inline]
    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
        self
    }
}

macro_rules! seq {
    (cap $seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<T> BorrowMut for $seq<T>
        where
            T: BorrowMut,
            $(for<'a> T::TargetMut<'a>: $trait,)*
        {
            type TargetMut<'a> = $seq<T::TargetMut<'a>> where T: 'a;

            #[inline]
            fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
                let mut out = <$seq<_>>::with_capacity(self.len());

                for value in self {
                    out.$insert(value.borrow_mut());
                }

                out
            }
        }
    };

    ($seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<T> BorrowMut for $seq<T>
        where
            T: BorrowMut,
            $(for<'a> T::TargetMut<'a>: $trait,)*
        {
            type TargetMut<'a> = $seq<T::TargetMut<'a>> where T: 'a;

            #[inline]
            fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
                let mut out = <$seq<_>>::new();

                for value in self {
                    out.$insert(value.borrow_mut());
                }

                out
            }
        }
    };
}

macro_rules! map {
    (cap $map:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<K, V> BorrowMut for $map<K, V>
        where
            K: Borrow,
            V: BorrowMut,
            $(for<'a> K::Target<'a>: $trait,)*
        {
            type TargetMut<'a> = $map<K::Target<'a>, V::TargetMut<'a>> where K: 'a, V: 'a;

            #[inline]
            fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
                let mut out = <$map<_, _>>::with_capacity(self.len());

                for (key, value) in self {
                    out.$insert(key.borrow(), value.borrow_mut());
                }

                out
            }
        }
    };

    ($map:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        impl<K, V> BorrowMut for $map<K, V>
        where
            K: Borrow,
            V: BorrowMut,
            $(for<'a> K::Target<'a>: $trait,)*
        {
            type TargetMut<'a> = $map<K::Target<'a>, V::TargetMut<'a>> where K: 'a, V: 'a;

            #[inline]
            fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
                let mut out = <$map<_, _>>::new();

                for (key, value) in self {
                    out.$insert(key.borrow(), value.borrow_mut());
                }

                out
            }
        }
    };
}

seq!(cap Vec, push);
seq!(LinkedList, push_back);

map!(cap HashMap, insert, Hash, Eq);
map!(BTreeMap, insert, PartialOrd, Ord, Eq);
