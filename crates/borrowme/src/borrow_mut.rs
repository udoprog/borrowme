#[cfg(feature = "std")]
mod std;

/// Borrow mutably from self.
///
/// This works similarly to [`BorrowMut`][std-borrow-mut] but allows borrowing
/// compoundly from `&self` by defining a [generic `TargetMut`]. This means it's
/// not just limited to returning an immediate reference to the borrowed value
/// but can return something which receives lifetime parameters that borrows
/// from self. This is called "compound borrowing" because it lets you return
/// types which contains compound references.
///
/// It is recommended that you use [`borrow_mut`][crate::borrow_mut()] instead
/// of importing this trait.
///
/// <br>
///
/// # What about `std::borrow::BorrowMut`?
///
/// The [`BorrowMut`][std-borrow-mut] trait as defined can't perform compound
/// borrows from `&mut self`. Because the `borrow_mut` method immediately
/// returns *a reference* to the borrowed type.
///
/// ```
/// # trait Borrow<Borrowed: ?Sized> { fn borrow(&self) -> &Borrowed; }
/// trait BorrowMut<Borrowed: ?Sized>: Borrow<Borrowed> {
///     fn borrow_mut(&mut self) -> &mut Borrowed;
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
/// compared to [`BorrowMut<T>`][std-borrow-mut]. But for our purposes this is
/// fine. This crate is primarily intended to work with two symmetrical types
/// and any deviation from that pattern can be handled by customizing the
/// behavior of the [`#[borrowme]`][crate::borrowme] attribute.
///
/// [std-borrow-mut]: ::std::borrow::BorrowMut
/// [generic `TargetMut`]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
pub trait BorrowMut {
    type TargetMut<'a>
    where
        Self: 'a;

    /// Borrow mutably from `self`.
    fn borrow_mut(&mut self) -> Self::TargetMut<'_>;
}

impl<T> BorrowMut for Option<T>
where
    T: BorrowMut,
{
    type TargetMut<'a>
        = Option<T::TargetMut<'a>>
    where
        T: 'a;

    #[inline]
    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
        self.as_mut().map(|some| some.borrow_mut())
    }
}

impl<T> BorrowMut for [T] {
    type TargetMut<'a>
        = &'a mut [T]
    where
        T: 'a;

    #[inline]
    fn borrow_mut(&mut self) -> Self::TargetMut<'_> {
        self
    }
}
