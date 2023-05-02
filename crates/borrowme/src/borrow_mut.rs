#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::collections::{BTreeMap, HashMap, LinkedList};

use crate::Borrow;

/// Borrow mutably from self.
///
/// This works similarly to [`BorrowMut`][std::borrow::BorrowMut] but allows
/// borrowing compoundly from `&self` by defining a [generic `TargetMut`]. This
/// means it's not just limited to returning an immediate reference to the
/// borrowed value but can return something which receives lifetime parameters
/// that borrows from self. This is called "compound borrowing" because it lets
/// you return types which contains compound references.
///
/// It is recommended that you use [`borrow_mut`][crate::borrow_mut()] instead
/// of importing this trait.
///
/// <br>
///
/// # What about `std::borrow::BorrowMut`?
///
/// The [`BorrowMut`][std::borrow::BorrowMut] trait as defined can't perform
/// compound borrows from `&mut self`. Because the `borrow_mut` method
/// immediately returns *a reference* to the borrowed type.
///
/// ```
/// # trait Borrow<Borrowed: ?Sized> { fn borrow(&self) -> &Borrowed; }
/// trait BorrowMut<Borrowed: ?Sized>: Borrow<Borrowed> {
///     fn borrow_mut(&mut self) -> &Borrowed;
/// }
/// ```
///
/// This means that there is no way to implement something like
/// `BorrowMut<Word<'a>>` because it's required that we return a reference which
/// doesn't outlive `'a`, something that can't be satisfied from the call to
/// `&mut self`.
///
/// ```compile_fail
/// # use std::borrow::BorrowMut;
/// struct WordMut<'a>(&'a mut String);
/// struct OwnedWord(String);
///
/// impl<'a> BorrowMut<WordMut<'a>> for OwnedWord {
///     fn borrow_mut(&mut self) -> &mut WordMut<'a> {
///         &mut WordMut(&mut self.0)
///     }
/// }
/// ```
///
/// ```text
/// error[E0515]: cannot return reference to temporary value
///  --> src/borrow.rs:37:9
///   |
/// 9 |         &mut WordMut(&mut self.0)
///   |         ^------------------------
///   |         ||
///   |         |temporary value created here
///   |         returns a reference to data owned by the current function
/// ```
///
/// The solution implemented in this crate is to use a [generic `TargetMut`], with
/// which we can implement `borrow_mut` like this:
///
/// ```
/// # struct WordMut<'a>(&'a mut String);
/// # struct OwnedWord(String);
/// use borrowme::BorrowMut;
///
/// impl BorrowMut for OwnedWord {
///     type TargetMut<'a> = WordMut<'a>;
///
///     fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
///         WordMut(&mut self.0)
///     }
/// }
/// ```
///
/// A catch here is that `BorrowMut` can only be implemented once for each time,
/// compared to [`BorrowMut<T>`][std::borrow::BorrowMut]. But for our purposes this is
/// fine. This crate is primarily intended to work with two symmetrical types
/// and any deviation from that pattern can be handled by customizing the
/// behavior of the [`#[borrowme]`][crate::borrowme] attribute.
///
/// [generic `TargetMut`]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
pub trait BorrowMut {
    type TargetMut<'a>
    where
        Self: 'a;

    /// Borrow mutably from `self`.
    fn borrow_mut(&mut self) -> Self::TargetMut<'_>;
}

#[cfg(feature = "std")]
impl BorrowMut for String {
    type TargetMut<'a> = &'a mut String;

    #[inline]
    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
        self
    }
}

impl<T> BorrowMut for Option<T>
where
    T: BorrowMut,
{
    type TargetMut<'a> = Option<T::TargetMut<'a>> where T: 'a;

    #[inline]
    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
        self.as_mut().map(|some| some.borrow_mut())
    }
}

impl<T> BorrowMut for [T] {
    type TargetMut<'a> = &'a mut [T] where T: 'a;

    #[inline]
    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
        self
    }
}

macro_rules! seq {
    (cap $seq:ident, $insert:ident $(, $trait:path)* $(,)?) => {
        #[cfg(feature = "std")]
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
        #[cfg(feature = "std")]
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
        #[cfg(feature = "std")]
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
        #[cfg(feature = "std")]
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
