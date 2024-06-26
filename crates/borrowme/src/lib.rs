//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/borrowme-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/borrowme)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/borrowme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/borrowme)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-borrowme-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/borrowme)
//!
//! The missing compound borrowing for Rust.
//!
//! Rust comes with two sibling traits which that can convert from owned to
//! borrowed: [`ToOwned`][std-to-owned], [`Borrow`][std-borrow] and
//! [`BorrowMut`][std-borrow-mut].
//!
//! These can convert most simple types such as `&str` to and from `String`. But
//! lets think of this in a broader perspective. How to we convert a type that
//! *has lifetimes*, to one which *does not*? This crate defines its own
//! [`ToOwned`], [`Borrow`] and [`BorrowMut`] traits which serve a similar
//! purpose to the ones in `std` but are implemented so that they can do this
//! not only for simple references but also for *compound types* which receives
//! lifetimes.
//!
//! To help us implement these traits the [`#[borrowme]`][borrowme] attribute
//! macro is provided ([see this section][borrowme-derive] for why it's not a
//! derive).
//!
//! ```
//! # use borrowme::borrowme;
//! #[borrowme]
//! #[derive(Clone)]
//! #[borrowed_attr(derive(Copy))]
//! struct Word<'a> {
//!     text: &'a str,
//! }
//! ```
//!
//! From this we get the following types and implementations:
//!
//! ```
//! #[derive(Clone, Copy)]
//! struct Word<'a> {
//!     text: &'a str,
//! }
//!
//! #[derive(Clone)]
//! struct OwnedWord {
//!     text: String,
//! }
//!
//! impl borrowme::ToOwned for Word<'_> {
//!     type Owned = OwnedWord;
//!
//!     fn to_owned(&self) -> OwnedWord {
//!         /* .. */
//!         # todo!()
//!     }
//! }
//!
//! impl borrowme::Borrow for OwnedWord {
//!     type Target<'a> = Word<'a>;
//!
//!     fn borrow(&self) -> Word<'_> {
//!         /* .. */
//!         # todo!()
//!     }
//! }
//! ```
//!
//! By itself this isn't much, but here's the big trick. Types using this crate
//! can be composed and converted into their borrowed or owned counterparts as
//! needed:
//!
//! ```
//! # use borrowme::borrowme;
//! use std::collections::HashMap;
//!
//! #[borrowme]
//! struct Word<'a> {
//!     text: &'a str,
//! }
//!
//! #[borrowme]
//! struct Dictionary<'a> {
//!     words: HashMap<&'a str, Word<'a>>,
//! }
//!
//! let dictionary = Dictionary {
//!     /* .. */
//!     # words: HashMap::new(),
//! };
//!
//! let owned_dictionary: OwnedDictionary = borrowme::to_owned(&dictionary);
//! let dictionary2: Dictionary<'_> = borrowme::borrow(&owned_dictionary);
//! ```
//!
//! <br>
//!
//! [`Borrow`]: https://docs.rs/borrowme/latest/borrowme/trait.Borrow.html
//! [`BorrowMut`]: https://docs.rs/borrowme/latest/borrowme/trait.BorrowMut.html
//! [`ToOwned`]: https://docs.rs/borrowme/latest/borrowme/trait.ToOwned.html
//! [borrowme-derive]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html#why-isnt-this-a-derive
//! [borrowme]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html
//! [generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
//! [std-borrow-mut]: https://doc.rust-lang.org/std/borrow/trait.BorrowMut.html
//! [std-borrow]: https://doc.rust-lang.org/std/borrow/trait.Borrow.html
//! [std-to-owned]: https://doc.rust-lang.org/std/borrow/trait.ToOwned.html

#![cfg_attr(not(feature = "std"), no_std)]

/// Automatically build an *owned* variant of a type and implement [`ToOwned`] and
/// [`Borrow`].
///
/// Anything captured by the macro will be forwarded to the generated variant.
/// To have detailed control over this behavior, see the
/// `#[borrowed_attr(<meta>)]` and `#[owned_attr(<meta>)]` attributes below.
///
/// In order to work as intended, `#[borrowme]` must be used *before* any
/// attributes that you want it to capture such as derives.
///
/// ```
/// # use borrowme::borrowme;
/// use serde::Serialize;
///
/// #[borrowme]
/// #[derive(Serialize)]
/// pub struct Word<'a> {
///     lang: &'a str,
/// }
///
/// # fn implements_serialize<T: serde::Serialize>() {}
/// implements_serialize::<Word<'_>>();
/// implements_serialize::<OwnedWord>();
/// ```
///
/// If we use the wrong order, `Serialize` won't be seen and that derive won't
/// be transferred to `OwnedWord`.
///
/// ```compile_fail
/// # use borrowme::borrowme;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// #[borrowme]
/// pub struct Word<'a> {
///     lang: &'a str,
/// }
///
/// # fn implements_serialize<T: serde::Serialize>() {}
/// implements_serialize::<Word<'_>>();
/// implements_serialize::<OwnedWord>();
/// ```
///
/// ```text
/// 15 | implements_serialize::<OwnedWord>();
///    |                        ^^^^^^^^^ the trait `Serialize` is not implemented for `OwnedWord`
/// ```
///
/// <br>
///
/// ## Implementing for asymmetric types
///
/// Some common types need special care because their [`ToOwned`] and [`Borrow`]
/// implementations are asymmetric, that is that the [`Borrow::Target`] does not
/// match the type that implements [`ToOwned`]. This is easily addressed by
/// overriding the borrow implementation with [`#[borrowme(borrow_with =
/// <path>)]`][borrow_with].
///
/// #### `&[T]`
///
/// The [`ToOwned`] implementation produces a `Vec<T>`, while [`Borrow`] of
/// `Vec<T>` produces a `Vec<&T::Target>`. This can be fixed using
/// [`Vec::as_slice`].
///
/// ```
/// use borrowme::borrowme;
///
/// #[borrowme]
/// struct VecField<'a> {
///     #[borrowme(borrow_with = Vec::as_slice)]
///     strings: &'a [String],
/// }
/// ```
///
/// <br>
///
/// #### `&[T]` as an owned `Box<[T]>`
///
/// The [`ToOwned`] implementation produces a `Vec<T>`, while borrowing that
/// produces a `Vec<&T::Target>`. This can be fixed using [`Box::from`].
///
/// ```
/// use borrowme::borrowme;
///
/// #[borrowme]
/// struct Bytes<'a> {
///     #[borrowme(owned = Box<[u8]>, to_owned_with = Box::from)]
///     bytes: &'a [u8],
/// }
/// ```
///
/// <br>
///
/// ## Why isn't this a derive?
///
/// A derive macro can't see other attributes than the ones it declares as its
/// own. While this is very useful to provide better encapsulation across macros
/// it would mean that derives and other attributes that are specified on the
/// *borrowed* variant can't be forwarded to the owned one without explicitly
/// doing it yourself. This would be tedious because most of the time it's the
/// behaviour you want.
///
/// This should hopefully illustrate the issue:
///
/// ```compile_fail
/// use borrowme::{ToOwned, Borrow};
///
/// #[derive(Debug, Clone, PartialEq, Eq, ToOwned)]
/// #[owned_attr(derive(Debug, Clone, PartialEq, Eq, Borrow))]
/// struct Word<'a> {
///     text: &'a str,
/// }
/// ```
///
/// <br>
///
/// ## Container attributes
///
/// This section documents supported container attributes:
///
/// * [`#[borrowme(std)]`][container-std] which acts as if
///   [`#[borrowme(std)]`][std] is applied to every field and variant in the
///   container by default.
/// * [`#[borrowme(name = <ident>)]`][name] which is used to change the name of
///   the generated *owned* variant.
/// * [`#[borrowed_attr(<meta>)]`][b-c] and [`#[owned_attr(<meta>)]`][o-c] which
///   are used to add custom attributes.
///
/// Container attributes are attributes which are added to a container, such as
/// a `struct` or an `enum`. See for example `#[borrowme(name = WordBuf)]`
/// below:
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme(name = WordBuf)]
/// struct Word<'a> {
/// # text: &'a str,
///     /* body */
/// }
///
/// #[borrowme(name = EnumBuf)]
/// enum Enum<'a> {
///     /* body */
/// # First { text: &'a str },
/// # Second { text: &'a str },
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(std)]` container attribute
///
/// This container attribute acts as if [`#[borrowme(std)]`][std] is applied to
/// every field or variant in the container.
///
/// Note that this defeats copy and reference heuristics.
///
/// ```
/// use borrowme::borrowme;
///
/// #[borrowme]
/// #[borrowme(std)]
/// struct StdStruct<'a> {
///     a: &'a String,
///     #[borrowme(copy)]
///     b: u32,
/// }
///
/// #[borrowme]
/// #[borrowme(std)]
/// enum StdEnum<'a> {
///     Variant {
///         a: &'a String,
///         #[borrowme(copy)]
///         b: u32,
///     }
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(name = <ident>)]` container attribute
///
/// This allows you to pick the name to use for the generated type. By default
/// this is the original identifier prefixed with `Owned`.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme(name = WordBuf)]
/// #[derive(Debug, PartialEq)]
/// struct Word<'a> {
///     text: &'a str,
/// }
///
/// let word = Word {
///     text: "Hello World",
/// };
///
/// let word2 = WordBuf {
///     text: String::from("Hello World"),
/// };
///
/// assert_eq!(borrowme::to_owned(&word), word2);
/// ```
///
/// <br>
///
/// #### `#[borrowed_attr(<meta>)]` container attribute
///
/// Apply the given `<meta>` as a container attribute, but only for the
/// *borrowed* variant.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[borrowed_attr(derive(Clone))]
/// struct Word<'a> {
///     text: &'a str,
/// }
///
/// let word = Word {
///     text: "Hello World"
/// };
///
/// let word2 = word.clone();
/// assert_eq!(word.text, word2.text);
/// ```
///
/// <br>
///
/// #### `#[owned_attr(<meta>)]` container attribute
///
/// Apply the given given `<meta>` as a container attribute, but only to the
/// *owned* variant.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[owned_attr(derive(Clone))]
/// struct Word<'a> {
///     text: &'a str,
/// }
///
/// let word = OwnedWord {
///     text: String::from("Hello World")
/// };
///
/// let word2 = word.clone();
/// assert_eq!(word.text, word2.text);
/// ```
///
/// <br>
///
/// ## Variant attributes
///
/// This section documents supported variant attributes:
///
/// * [`#[borrowed_attr(<meta>)]`][b-v] and [`#[owned_attr(<meta>)]`][o-v] which
///   are used to add custom attributes.
/// * [`#[borrowme(std)]`][variant-std] which acts as if
///   [`#[borrowme(std)]`][std] is applied to every field in the variant by
///   default.
///
/// Variant attributes are attributes which apply to `enum` variants.
///
/// <br>
///
/// #### `#[borrowed_attr(<meta>)]` variant attribute
///
/// Apply the given `<meta>` as a variant attribute, but only for the *borrowed*
/// variant.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[borrowed_attr(derive(Default))]
/// enum Word<'a> {
///     Wiktionary(&'a str),
///     #[borrowed_attr(default)]
///     Unknown,
/// }
///
/// let word = Word::default();
/// assert!(matches!(word, Word::Unknown));
/// ```
///
/// <br>
///
/// #### `#[owned_attr(<meta>)]` variant attribute
///
/// Apply the given `<meta>` as a variant attribute, but only for the *owned*
/// variant.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[owned_attr(derive(Default))]
/// enum Word<'a> {
///     Wiktionary(&'a str),
///     #[owned_attr(default)]
///     Unknown,
/// }
///
/// let word = OwnedWord::default();
/// assert!(matches!(word, OwnedWord::Unknown));
/// ```
///
/// <br>
///
/// #### `#[borrowme(std)]` variant attribute
///
/// This container attribute acts as if [`#[borrowme(std)]`][std] is applied to
/// every field inside of a variant.
///
/// Note that this defeats copy and reference heuristics.
///
/// ```
/// use borrowme::borrowme;
///
/// #[borrowme]
/// enum StdEnum<'a> {
///     #[borrowme(std)]
///     Variant {
///         a: &'a String,
///         #[borrowme(copy)]
///         b: u32,
///     }
/// }
/// ```
///
/// <br>
///
///
/// <br>
///
/// ## Field attributes
///
/// This section documents supported field attributes:
///
/// * [`#[owned(<type>)]` or `#[borrowme(owned = <type>)]`][owned] which is a
///   required attribute for specifying the owned type a field is being
///   converted into.
/// * [`#[borrowme(mut)]`][mut] to indicate that the field needs mutable access
///   to the container.
/// * [`#[borrowme(to_owned_with = <path>)]`][to_owned_with],
///   [`#[borrowme(borrow_with = <path>)]`][borrow_with], and [`#[borrowme(with
///   = <path>)]`][with] which are used for customizing behavior.
/// * [`#[copy]` and `#[no_copy]`][copy] which is used to indicate if a field is
///   `Copy` and does not require conversion.
/// * [`#[borrowme(std)]`][std] which indicates that the field supports std-like
///   operations.
/// * [`#[borrowed_attr(<meta>)]`][b-f] and [`#[owned_attr(<meta>)]`][o-f] which
///   are used to add custom attributes.
///
/// Field attributes are attributes which apply to fields, such as the fields in
/// a struct.
///
/// <br>
///
/// #### `#[owned(<type>)]` or `#[borrowme(owned = <type>)]` field attributes
///
/// This specifies the owned type of the field. The latter variation is
/// available so that it looks better when combined with other attributes.
///
/// By default we try to automatically figure out the type through
/// `ToOwned::Owned` by converting the field type into an expression such as
/// `<<&'static str> as ::borrowme::ToOwned>::Owned`. When this doesn't work as
/// expected like when using a type which does not implement `ToOwned` this can
/// be overriden using this attribute.
///
/// ```
/// struct MyType;
/// struct MyOwnedType;
///
/// # use borrowme::borrowme;
/// #[borrowme]
/// pub struct Word<'a> {
///     #[borrowme(owned = MyOwnedType, to_owned_with = to_owned_my_type, borrow_with = borrow_my_type)]
///     lang: &'a MyType,
/// }
///
/// fn to_owned_my_type(this: &MyType) -> MyOwnedType {
///     MyOwnedType
/// }
///
/// fn borrow_my_type(this: &MyOwnedType) -> &MyType {
///     &MyType
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(mut)]` field attribute
///
/// Indicates that the field required mutable access to the parent container.
///
/// By default this uses heuristics. If a `&mut T` reference is noticed in the
/// field type mutable access is assumed.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// pub struct Text<'a> {
///     text: &'a mut String,
/// }
///
/// #[borrowme]
/// pub struct Word<'a> {
///     #[borrowme(mut)]
///     text: Text<'a>,
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(to_owned_with = <path>)]` field attribute
///
/// Specifies a path to use when making a field owned. By default this is:
/// * [`Clone`] if `#[borrowme(std)]` is specified.
/// * An owned `self.<field>` expression if `#[copy]` is specified.
/// * `::borrowme::ToOwned::to_owned`.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[derive(Clone, Debug)]
/// pub struct Word<'a> {
///     #[borrowme(owned = Option<String>, to_owned_with = option_to_owned)]
///     lang: Option<&'a str>,
/// }
///
/// #[inline]
/// pub(crate) fn option_to_owned(option: &Option<&str>) -> Option<String> {
///     option.map(ToOwned::to_owned)
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(borrow_with = <path>)]` field attribute
///
/// Specifies a path to use when borrowing a field. By default this is:
/// * A borrowed `&self.<field>` if `#[borrowme(std)]` is specified and the
///   field is not mutable.
/// * An owned `self.<field>` expression if `#[copy]` is specified.
/// * `::borrowme::Borrow::borrow`.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// pub struct Word<'a> {
///     #[borrowme(owned = Option<String>, borrow_with = option_borrow)]
///     lang: Option<&'a str>,
///     // Note that the above works the same as `Option::as_deref`.
///     #[borrowme(owned = Option<String>, borrow_with = Option::as_deref)]
///     lang2: Option<&'a str>,
/// }
///
/// #[inline]
/// pub(crate) fn option_borrow(option: &Option<String>) -> Option<&str> {
///     option.as_deref()
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(borrow_mut_with = <path>)]` field attribute
///
/// Using this implies `#[borrowme(mut)]`.
///
/// Specifies a path to use when borrowing a mutable field. By default this is:
/// * A borrowed `&mut self.<field>` if `#[borrowme(std)]` is specified and a
///   mutable field is detected or specified with `#[borrowme(mut)]`.
/// * An owned `self.<field>` expression if `#[copy]` is specified.
/// * `::borrowme::BorrowMut::borrow_mut`.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// pub struct Word<'a> {
///     #[borrowme(owned = Option<String>, borrow_mut_with = option_borrow)]
///     lang: Option<&'a mut str>,
///     #[borrowme(owned = Option<String>, borrow_mut_with = Option::as_deref_mut)]
///     lang2: Option<&'a mut str>,
/// }
///
/// #[inline]
/// pub(crate) fn option_borrow(option: &mut Option<String>) -> Option<&mut str> {
///     option.as_deref_mut()
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(with = <path>)]` field attribute
///
/// Specifies a path to use when calling `to_owned` and `borrow` on a field.
///
/// The sets `to_owned` to `<path>::to_owned`, and `borrow` to `<path>::borrow`.
///
/// Unless `#[copy]` or `#[borrowme(std)]` are specified, these are by
/// default:
/// * `::borrowme::ToOwned::to_owned`
/// * `::borrowme::Borrow::borrow`.
///
/// ```
/// # mod interior {
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[derive(Clone, Debug)]
/// pub struct Word<'a> {
///     #[owned(String)]
///     text: &'a str,
///     #[borrowme(owned = Option<String>, with = self::option)]
///     lang: Option<&'a str>,
/// }
///
/// pub(crate) mod option {
///     use borrowme::{Borrow, ToOwned};
///
///     #[inline]
///     pub(crate) fn borrow<T>(this: &Option<T>) -> Option<T::Target<'_>>
///     where
///         T: Borrow,
///     {
///         match this {
///             Some(some) => Some(some.borrow()),
///             None => None,
///         }
///     }
///
///     #[inline]
///     pub(crate) fn to_owned<T>(option: &Option<T>) -> Option<T::Owned>
///     where
///         T: ToOwned,
///     {
///         option.as_ref().map(ToOwned::to_owned)
///     }
/// }
/// # }
/// ```
///
/// <br>
///
/// #### `#[copy]` and `#[no_copy]` field attribute
///
/// These can also be specified as `#[borrowme(copy)]` and
/// `#[borrowme(no_copy)]`.
///
/// Indicates that the field type is `Copy`, if this is set then the value is
/// not cloned when the type is converted to and from its *owned* variant.
///
/// ```
/// # use borrowme::borrowme;
/// #[derive(Clone, Copy)]
/// struct OwnedBool(bool);
///
/// #[borrowme]
/// pub struct Word<'a> {
///     text: &'a str,
///     #[copy]
///     teineigo: OwnedBool,
/// }
/// ```
///
/// By default the macro performs heuristic to determine if a field is `Copy` or
/// not. This means that prelude types which make up common copy configurations
/// will be treated as copy. If this happens inadvertently the `#[no_copy]` or
/// `#[borrowme(std)]` attributes can be used.
///
/// Type which are `#[copy]` by default are:
/// * `u8`, `u16`, `u32`, `u64`, `u128`, and `usize`.
/// * `i8`, `i16`, `i32`, `i64`, `i128`, and `isize`.
/// * `f32` and `f64`.
/// * `bool`.
/// * Tuple types `(A, B, ..)` for which all of its elements look like they are
///   copy.
/// * Array types `[T; N]` for which the element `T` looks like they are copy.
///
/// This heuristic can be defeated in a handful of ways, depending on what best
/// suits your needs.
///
/// A field can specify that the type is not `Copy` using `#[no_copy]`, which
/// makes it fall back to the default behavior:
///
/// ```
/// # use borrowme::borrowme;
/// # #[allow(non_camel_case_types)]
/// struct bool;
///
/// impl borrowme::ToOwned for bool {
///     type Owned = bool;
///
///     fn to_owned(&self) -> Self::Owned {
///         bool
///     }
/// }
///
/// impl borrowme::Borrow for bool {
///     type Target<'a> = &'a bool;
///
///     fn borrow(&self) -> Self::Target<'_> {
///         self
///     }
/// }
///
/// #[borrowme]
/// pub struct Word<'a> {
///     #[no_copy]
///     teineigo: &'a bool,
/// }
/// ```
///
/// The field can specify `#[borrowme(std)]` to make use of standard methods of
/// cloning and getting a reference:
///
/// ```
/// # use borrowme::borrowme;
/// # #[allow(non_camel_case_types)]
/// #[derive(Clone)]
/// struct bool;
///
/// #[borrowme]
/// pub struct NoCopy<'a> {
///     #[borrowme(std)]
///     teineigo: &'a bool,
/// }
/// ```
///
/// Finally we can specify everything ourselves:
///
/// ```
/// # use borrowme::borrowme;
/// # #[allow(non_camel_case_types)]
/// #[derive(Clone)]
/// struct bool;
///
/// impl AsRef<bool> for bool {
///     fn as_ref(&self) -> &bool {
///         self
///     }
/// }
///
/// #[borrowme]
/// pub struct Word<'a> {
///     #[borrowme(owned = bool, borrow_with = AsRef::as_ref, to_owned_with = Clone::clone)]
///     teineigo: &'a bool,
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(std)]` field attribute
///
/// Causes conversion to happen by using the [`Clone`] trait to convert into an
/// owned type and a reference expression like `&self.<field>` to borrow,
/// mimicking the behaviour of `std::borrow`.
///
/// If the field is an immediate type behind a reference, that will be used as
/// the target *unless* `#[borrowme(owned = <type>)]` is specified.
///
/// ```
/// # use borrowme::borrowme;
/// # #[allow(non_camel_case_types)]
/// #[derive(Clone)]
/// struct WordKind;
///
/// #[borrowme]
/// pub struct Word<'a> {
///     #[borrowme(std)]
///     kind: &'a WordKind,
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowed_attr(<meta>)]` field attribute
///
/// Apply the given `<meta>` as a field attribute, but only for the *borrowed*
/// variant. This allows certain attributes that are only needed for the
/// borrowed variant to be implemented, such as `#[serde(borrow)]`.
///
/// ```
/// # use borrowme::borrowme;
/// use serde::{Serialize, Deserialize};
///
/// #[borrowme]
/// #[derive(Serialize, Deserialize)]
/// pub struct Word<'a> {
///     #[borrowed_attr(serde(borrow))]
///     lang: Option<&'a str>,
/// }
/// ```
///
/// <br>
///
/// #### `#[owned_attr(<meta>)]` field attribute
///
/// Apply the given `<meta>` as a field attribute, but only for the *owned*
/// variant.
///
/// In the below example, only the *owned* variant will have a serde
/// implementation:
///
/// ```
/// # use borrowme::borrowme;
/// use serde::{Serialize, Deserialize};
///
/// #[borrowme]
/// #[owned_attr(derive(Serialize, Deserialize))]
/// struct Word<'a> {
///     #[owned_attr(serde(default, skip_serializing_if = "Option::is_none"))]
///     lang: Option<&'a str>,
/// }
/// ```
///
/// [b-c]: #borrowed_attrmeta-container-attribute
/// [b-f]: #borrowed_attrmeta-field-attribute
/// [b-v]: #borrowed_attrmeta-variant-attribute
/// [borrow_with]: #borrowmeborrow_with--path-field-attribute
/// [container-std]: #borrowmestd-container-attribute
/// [copy]: #copy-and-no_copy-field-attribute
/// [mut]: #borrowmemut-field-attribute
/// [name]: #borrowmename--ident-container-attribute
/// [o-c]: #owned_attrmeta-container-attribute
/// [o-f]: #owned_attrmeta-field-attribute
/// [o-v]: #owned_attrmeta-variant-attribute
/// [owned]: #ownedtype-or-borrowmeowned--type-field-attributes
/// [std]: #borrowmestd-field-attribute
/// [to_owned_with]: #borrowmeto_owned_with--path-field-attribute
/// [variant-std]: #borrowmestd-variant-attribute
/// [with]: #borrowmewith--path-field-attribute
#[doc(inline)]
pub use borrowme_macros::borrowme;

mod borrow;
pub use self::borrow::Borrow;

mod borrow_mut;
pub use self::borrow_mut::BorrowMut;

mod to_owned;
pub use self::to_owned::ToOwned;

/// Convert a value to owned.
///
/// This helper function is provided so that you don't have to have the
/// [`ToOwned`] trait in scope, and make it explicit when this crate is being
/// used since this conversion is not a cheap operation in this crate.
///
/// Using this also prevents conflicts with the built-in
/// [`std::borrow::ToOwned`] which is in the prelude.
///
/// <br>
///
/// # Examples
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// struct Word<'a> {
///     text: &'a str,
/// }
///
/// impl OwnedWord {
///     fn new(text: &str) -> Self {
///         Self { text: text.to_owned() }
///     }
/// }
///
/// #[borrowme]
/// #[derive(Default)]
/// struct Dictionary<'a> {
///     words: Vec<Word<'a>>,
/// }
///
/// fn uppercase(dictionary: OwnedDictionary) -> Vec<String> {
///     let mut out = Vec::new();
///
///     for word in dictionary.words {
///         out.push(word.text.to_uppercase());
///     }
///
///     out
/// }
///
/// let mut dictionary = Dictionary::default();
/// dictionary.words.push(Word { text: "Hello" });
/// dictionary.words.push(Word { text: "World" });
///
/// let out = uppercase(borrowme::to_owned(dictionary));
///
/// assert_eq!(out[0], "HELLO");
/// assert_eq!(out[1], "WORLD");
/// ```
#[inline]
pub fn to_owned<T>(value: T) -> T::Owned
where
    T: ToOwned,
{
    value.to_owned()
}

/// Borrow from the given value.
///
/// This helper function is provided so that you don't have to have the
/// [`Borrow`] trait in scope, and make it explicit when this crate is being
/// used since "borrowing" is not a cheap operation in this crate.
///
/// This also prevents conflicts with the built-in
/// [`Borrow`][std::borrow::Borrow].
///
/// <br>
///
/// # Examples
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// struct Word<'a> {
///     text: &'a str,
/// }
///
/// impl OwnedWord {
///     fn new(text: &str) -> Self {
///         Self { text: text.to_owned() }
///     }
/// }
///
/// #[borrowme]
/// #[derive(Default)]
/// struct Dictionary<'a> {
///     words: Vec<Word<'a>>,
/// }
///
/// fn uppercase(dictionary: Dictionary<'_>) -> Vec<String> {
///     let mut out = Vec::new();
///
///     for word in dictionary.words {
///         out.push(word.text.to_uppercase());
///     }
///
///     out
/// }
///
/// let mut dictionary = OwnedDictionary::default();
/// dictionary.words.push(OwnedWord::new("Hello"));
/// dictionary.words.push(OwnedWord::new("World"));
///
/// let out = uppercase(borrowme::borrow(&dictionary));
///
/// assert_eq!(out[0], "HELLO");
/// assert_eq!(out[1], "WORLD");
/// ```
#[inline]
pub fn borrow<T>(value: &T) -> T::Target<'_>
where
    T: ?Sized + Borrow,
{
    value.borrow()
}

/// Borrow mutably from the given value.
///
/// This helper function is provided so that you don't have to have the
/// [`BorrowMut`] trait in scope, and make it explicit when this crate is being
/// used since "borrowing" is not a cheap operation in this crate.
///
/// This also prevents conflicts with the built-in
/// [`BorrowMut`][std::borrow::BorrowMut].
///
/// <br>
///
/// # Examples
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// struct Word<'a> {
///     text: &'a mut String,
/// }
///
/// impl OwnedWord {
///     fn new(text: &str) -> Self {
///         Self { text: text.to_owned() }
///     }
/// }
///
/// #[borrowme]
/// #[derive(Default)]
/// struct Dictionary<'a> {
///     #[borrowme(mut)]
///     words: Vec<Word<'a>>,
/// }
///
/// fn uppercase(dictionary: Dictionary<'_>) {
///     for word in dictionary.words {
///         *word.text = word.text.to_uppercase();
///     }
/// }
///
/// let mut dictionary = OwnedDictionary::default();
/// dictionary.words.push(OwnedWord::new("Hello"));
/// dictionary.words.push(OwnedWord::new("World"));
///
/// uppercase(borrowme::borrow_mut(&mut dictionary));
///
/// assert_eq!(dictionary.words[0].text, "HELLO");
/// assert_eq!(dictionary.words[1].text, "WORLD");
/// ```
#[inline]
pub fn borrow_mut<T>(value: &mut T) -> T::TargetMut<'_>
where
    T: ?Sized + BorrowMut,
{
    value.borrow_mut()
}
