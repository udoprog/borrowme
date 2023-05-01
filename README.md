# borrowme

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/borrowme-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/borrowme)
[<img alt="crates.io" src="https://img.shields.io/crates/v/borrowme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/borrowme)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-borrowme-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/borrowme)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/borrowme/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/borrowme/actions?query=branch%3Amain)

The missing compositional borrowing for Rust.

This crate provides an attribute macro which helps you achieve compositional
borrowing. Roughly this means that you can convert a struct which has
lifetimes into ones which does not and vice versa.

> **Note:** See the [`#[borrowme]`][borrowme] attribute for more
> documentation.

```rust
#[borrowme::borrowme]
#[derive(Debug, PartialEq, Eq)]
struct Word<'a> {
    #[owned(ty = String)]
    text: &'a str,
    #[owned(ty = Option<String>)]
    lang: Option<&'a str>,
}

let text = String::from("Hello World");
let lang = Some(String::from("eng"));

let word = Word {
    text: "hello",
    lang: lang.as_deref(),
};

let owned_word: OwnedWord = borrowme::to_owned(&word);
assert_eq!(&owned_word.text, word.text);
assert_eq!(owned_word.lang.as_deref(), word.lang);

let word2: Word<'_> = borrowme::borrow(&owned_word);
assert_eq!(word2, word);
```

<br>

## Why can't we use [`std::borrow::ToOwned`][std-to-owned] and [`std::borrow::Borrow`][std-borrow]?

Rust comes with two sibling traits which both are responsible for converting
something to an owned and a borrowed variant: [`ToOwned`][std-to-owned] and
[`Borrow`][std-borrow].

These convert a type to a *borrowed* value to an owned one, let's think
about it from a broader perspective: How to we convert a type which *has
lifetimes*, to one which *does not*?

```rust
struct Word<'a> {
    text: &'a str,
    lang: Option<&'a str>,
}
```

Let's try to implement `ToOwned` for this type.

```compile_fail
# struct Word<'a> { text: &'a str, lang: Option<&'a str> }
struct OwnedWord {
    text: String,
    lang: Option<String>,
}

impl ToOwned for Word<'_> {
    type Owned = OwnedWord;

    #[inline]
    fn to_owned(&self) -> OwnedWord {
        OwnedWord {
            text: self.text.to_owned(),
            lang: self.lang.map(ToOwned::to_owned),
        }
    }
}
```

```text
error[E0277]: the trait bound `OwnedWord: std::borrow::Borrow<Word<'_>>` is not satisfied
  --> src/lib.rs:27:18
   |
11 |     type Owned = OwnedWord;
   |                  ^^^^^^^^^ the trait `std::borrow::Borrow<Word<'_>>` is not implemented for `OwnedWord`
   |
note: required by a bound in `std::borrow::ToOwned::Owned`
  --> alloc/src/borrow.rs:41:17
   |
41 |     type Owned: Borrow<Self>;
   |                 ^^^^^^^^^^^^ required by this bound in `ToOwned::Owned`
```

This happens because [`ToOwned`][std-to-owned] is a symmetric trait, which
requires that the resulting `Owned` type can be borrowed back into the type
being converted.

So the first task is to define a new [`ToOwned`] trait which does not
require the produced value to be [`Borrow`][std-borrow].

Simple enough, but what if we need to go *the other* way?

The [`Borrow`][std-borrow] trait as defined faces an issue which can't be
easily addressed. The `borrow` method returns *a reference* to the borrowed
type.

```rust
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
```

This means that there is no way to implement `Borrow<Word<'a>>` because it
required that we return a reference which doesn't outlive `'a`, something
that can't be satisfied because we don't hold a reference to `Word<'a>`.

```compile_fail
# use std::borrow::Borrow;
# struct Word<'a> { text: &'a str, lang: Option<&'a str> }
# struct OwnedWord { text: String, lang: Option<String> }
impl<'a> Borrow<Word<'a>> for OwnedWord {
    fn borrow(&self) -> &Word<'a> {
        &Word {
           text: self.text.as_str(),
           lang: self.lang.as_ref().map(String::as_str),
        }
    }
}
```

```text
error: lifetime may not live long enough
  --> src/lib.rs:83:9
   |
6  |   impl<'a> Borrow<Word<'a>> for OwnedWord {
   |        -- lifetime `'a` defined here
7  |       fn borrow(&self) -> &Word<'a> {
   |                 - let's call the lifetime of this reference `'1`
8  | /         &Word {
9  | |            text: self.text.as_str(),
10 | |            lang: self.lang.as_ref().map(String::as_str),
11 | |         }
   | |_________^ associated function was supposed to return data with lifetime `'a` but it is returning data with lifetime `'1`
```

The solution this crate provides is to define a new [`Borrow`] trait which
makes use of [generic associated types]. This allows it to structurally
decompose a borrowed value.

```rust
pub trait Borrow {
    type Target<'a>
    where
        Self: 'a;

    fn borrow(&self) -> Self::Target<'_>;
}
```

Now we can implement it for `OwnedWord`:

```rust
impl Borrow for OwnedWord {
    type Target<'a> = Word<'a>;

    fn borrow(&self) -> Self::Target<'_> {
        Word {
           text: self.text.as_str(),
           lang: self.lang.as_ref().map(String::as_str),
        }
    }
}
```

The catch here is that `Borrow` can only be implemented once for each time,
compared to [`Borrow<T>`][std-borrow]. But for our purposes this is fine.
This crate is primarily intended to work with two symmetrical types.

[`ToOwned`]: https://docs.rs/borrowme
[generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
[borrowme]: https://docs.rs/borrowme
[std-borrow]: std::borrow::Borrow
[std-to-owned]: std::borrow::ToOwned
