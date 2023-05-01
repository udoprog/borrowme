//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/borrowme-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/borrowme)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/borrowme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/borrowme)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-borrowme-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/borrowme)
//!
//! The missing compositional borrowing for Rust.
//!
//! This crate provides an attribute macro which helps you pair two types
//! through compositional borrowing and ownership conversion. Roughly this means
//! that you can convert a struct which has lifetimes into ones which does not
//! and such as between the `Word` and `OwnedWord` structs here:
//!
//! ```
//! #[derive(Debug, PartialEq, Eq)]
//! struct Word<'a> {
//!     text: &'a str,
//!     lang: Option<&'a str>,
//!     examples: Vec<&'a str>,
//! }
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct OwnedWord {
//!     text: String,
//!     lang: Option<String>,
//!     examples: Vec<String>,
//! }
//! ```
//!
//! Writing and maintaining the `OwnedWord` variant is labour intensive and
//! error prone. Instead we can use the [`#[borrowme]`][borrowme] attribute
//! provided by this crate:
//!
//! ```
//! use borrowme::borrowme;
//!
//! #[borrowme]
//! #[derive(Debug, PartialEq, Eq)]
//! struct Word<'a> {
//!     text: &'a str,
//!     lang: Option<&'a str>,
//!     examples: Vec<&'a str>,
//! }
//! ```
//!
//! See the [`#[borrowme]`][borrowme] attribute for detailed documentation on
//! how the attribute works.
//!
//! ```
//! # #[borrowme::borrowme] #[derive(Debug, PartialEq, Eq)] struct Word<'a> {
//! # text: &'a str, lang: Option<&'a str>, examples: Vec<&'a str>,
//! # }
//! let text = String::from("Hello");
//! let lang = Some(String::from("eng"));
//! let examples = vec![String::from("Hello World")];
//!
//! let word = Word {
//!     text: "Hello World",
//!     lang: lang.as_deref(),
//!     examples: examples.iter().map(|s| s.as_str()).collect(),
//! };
//!
//! let word2: OwnedWord = borrowme::to_owned(&word);
//! let word3: Word<'_> = borrowme::borrow(&word2);
//! assert_eq!(word3, word);
//! ```
//!
//! <br>
//!
//! Rust comes with two sibling traits which both are responsible for converting
//! something to an owned and a borrowed variant: [`ToOwned`][std-to-owned] and
//! [`Borrow`][std-borrow].
//!
//! These convert a type to a *borrowed* value to an owned one, let's think
//! about it from a broader perspective: How to we convert a type which *has
//! lifetimes*, to one which *does not*?
//!
//! To this end this crate defines two similar traits: [`ToOwned`] and
//! [`Borrow`]. These traits serve a similar purpose to the traits in `std` but
//! are implemented differently. See their corresponding documentation for more
//! details.
//!
//! [`ToOwned`]: https://docs.rs/borrowme/latest/borrowme/trait.ToOwned.html
//! [generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
//! [borrowme]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html
//! [std-borrow]: std::borrow::Borrow
//! [std-to-owned]: std::borrow::ToOwned

/// Automatically build an *owned* variant of a type and implement [`ToOwned`] and
/// [`Borrow`].
///
/// Anything captured by the macro will be forwarded to the generated variant.
/// To have detailed control over this behavior, see the `#[owned_attr(<meta>)]`
/// and `#[owned_attr(<meta>)]` attributes below.
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
/// ## Container attributes
///
/// The following section documents supported container attributes:
///
/// * [`#[borrowme(prefix = <ident>)]`][prefix] which is used to change the prefix of the
///   generated *owned* variant.
/// * [`#[borrowed_attr(<meta>)]`][borrowed_attr] and
///   [`#[owned_attr(<meta>)]`][owned_attr] which are used to add custom
///   attributes.
///
/// [prefix]: #borrowmeprefix--ident-container-attribute
/// [borrowed_attr]: #borrowed_attrmeta-container-attribute
/// [owned_attr]: #owned_attrmeta-container-attribute
///
/// Container attributes are attributes which are added to a container, such as
/// a `struct` or an `enum`. See for example `#[borrowme(prefix = Prefix)]`
/// below:
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme(prefix = Prefix)]
/// struct Struct<'a> {
/// # text: &'a str,
///     /* body */
/// }
///
/// #[borrowme(prefix = Prefix)]
/// enum Enum<'a> {
///     /* body */
/// # First { text: &'a str },
/// # Second { text: &'a str },
/// }
/// ```
///
/// <br>
///
/// #### `#[borrowme(prefix = <ident>)]` container attribute
///
/// This allows you to pick the prefix to use for the generated type. By default
/// this is `Owned`.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme(prefix = Prefix)]
/// #[derive(Debug, PartialEq)]
/// struct Word<'a> {
///     text: &'a str,
/// }
///
/// let word = Word {
///     text: "Hello World",
/// };
///
/// let word2 = PrefixWord {
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
/// The following section documents supported variant attributes:
///
/// * [`#[borrowed_attr(<meta>)]`][borrowed_attr] and
///   [`#[owned_attr(<meta>)]`][owned_attr] which are used to add custom
///   attributes.
///
/// Variant attributes are attributes which apply to `enum` variants.
///
/// [borrowed_attr]: #borrowed_attrmeta-variant-attribute
/// [owned_attr]: #owned_attrmeta-variant-attribute
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
/// ## Field attributes
///
/// The following section documents supported field attributes:
///
/// * [`#[owned(<type>)]` or `#[selectme(owned = <type>)]`][owned] which is a
///   required attribute for specifying the owned type a field is being
///   converted into.
/// * [`#[borrowme(to_owned_with = <path>)]`][to_owned_with],
///   [`#[borrowme(borrow_with = <path>)]`][borrow_with], and [`#[borrowme(with
///   = <path>)]`][with] which are used for customizing behavior.
/// * [`#[copy]`][copy] which is used to indicate that the field is `Copy` and
///   does not require conversion.
/// * [`#[borrowed_attr(<meta>)]`][borrowed_attr] and
///   [`#[owned_attr(<meta>)]`][owned_attr] which are used to add custom
///   attributes.
///
/// Field attributes are attributes which apply to fields, such as the fields in
/// a struct.
///
/// <br>
///
/// [owned]: #ownedtype-or-selectmeowned--type-field-attributes
/// [to_owned_with]: #borrowmeto_owned_with--path-field-attribute
/// [borrow_with]: #borrowmeborrow_with--path-field-attribute
/// [with]: #borrowmewith--path-field-attribute
/// [copy]: #copy-field-attribute
/// [borrowed_attr]: #borrowed_attrmeta-field-attribute
/// [owned_attr]: #owned_attrmeta-field-attribute
///
/// #### `#[owned(<type>)]` or `#[selectme(owned = <type>)]` field attributes
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
/// #### `#[borrowme(to_owned_with = <path>)]` field attribute
///
/// Specifies a path to use when making a field owned. By default this is:
/// * `::core::clone::Clone::clone` if `#[owned(<type>)]` is not specified.
/// * `::borrowme::ToOwned::to_owned` if `#[owned(<type>)]` is specified.
/// * An owned `self.<field>` expression if `#[copy]` is specified.
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
/// Specifies a path to use when making a field owned. By default this is:
/// * A borrowed `&self.<field>` if `#[owned(<type>)]` is not specified.
/// * `::borrowme::Borrowed::borrow` if `#[owned(<type>)]` is specified.
/// * An owned `self.<field>` expression if `#[copy]` is specified.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// #[derive(Clone, Debug)]
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
/// #### `#[borrowme(with = <path>)]` field attribute
///
/// Specifies a path to use when calling `to_owned` and `borrow` on a field.
///
/// The sets `to_owned` to `<path>::to_owned`, and `borrow` to `<path>::borrow`.
/// By default these are otherwise `::borrowme::ToOwned::to_owned` and
/// `::borrowme::Borrowed::borrow` unless `#[owned(copy)]` is specified.
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
/// #### `#[copy]` field attribute
///
/// Indicates that the field type is `Copy`, if this is set then the value is
/// not cloned when the type is converted to and from its *owned* variant.
///
/// ```
/// # use borrowme::borrowme;
/// #[borrowme]
/// pub struct Word<'a> {
///     #[owned(String)]
///     text: &'a str,
///     #[copy]
///     teineigo: bool,
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
#[doc(inline)]
pub use borrowme_macros::borrowme;

mod borrow;
pub use self::borrow::Borrow;

mod to_owned;
pub use self::to_owned::ToOwned;

/// Convert the value into an *owned* variant.
///
/// This helper function is provided so that you don't have to have the
/// [`ToOwned`] trait in scope, and make it explicit when this crate is being
/// used since this conversion is not a cheap operation in this crate.
///
/// This also prevents conflicts with the built-in
/// [`ToOwned`][std::borrow::ToOwned].
pub fn to_owned<T>(value: T) -> T::Owned
where
    T: ToOwned,
{
    value.to_owned()
}

/// Borrow the given value.
///
/// This helper function is provided so that you don't have to have the [`Borrow`]
/// trait in scope, and make it explicit when this crate is being used since
/// "borrowing" is not a cheap operation in this crate.
///
/// This also prevents conflicts with the built-in
/// [`Borrow`][std::borrow::Borrow].
pub fn borrow<T>(value: &T) -> T::Target<'_>
where
    T: ?Sized + Borrow,
{
    value.borrow()
}
