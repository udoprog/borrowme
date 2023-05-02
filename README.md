# borrowme

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/borrowme-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/borrowme)
[<img alt="crates.io" src="https://img.shields.io/crates/v/borrowme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/borrowme)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-borrowme-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/borrowme)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/borrowme/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/borrowme/actions?query=branch%3Amain)

The missing compound borrowing for Rust.

Rust comes with two sibling traits which that can convert from owned to
borrowed: [`ToOwned`][std-to-owned], [`Borrow`][std-borrow] and
[`BorrowMut`][std-borrow-mut].

These can convert most simple types such as `&str` to and from `String`. But
lets think of this in a broader perspective. How to we convert a type that
*has lifetimes*, to one which *does not*? This crate defines its own
[`ToOwned`], [`Borrow`] and [`BorrowMut`] traits which serve a similar
purpose to the ones in `std` but are implemented so that they can do this
not only for simple references but also for *compound types* which receives
lifetimes.

To help us implement these traits the [`#[borrowme]`][borrowme] attribute
macro is provided ([see this section][borrowme-derive] for why it's not a
derive).

```rust
#[borrowme]
#[derive(Clone)]
#[borrowed_attr(derive(Copy))]
struct Word<'a> {
    text: &'a str,
}
```

From this we get the following types and implementations:

```rust
#[derive(Clone, Copy)]
struct Word<'a> {
    text: &'a str,
}

#[derive(Clone)]
struct OwnedWord {
    text: String,
}

impl borrowme::ToOwned for Word<'_> {
    type Owned = OwnedWord;

    fn to_owned(&self) -> OwnedWord {
        /* .. */
    }
}

impl borrowme::Borrow for OwnedWord {
    type Target<'a> = Word<'a>;

    fn borrow(&self) -> Word<'_> {
        /* .. */
    }
}
```

By itself this isn't much, but here's the big trick. Types using this crate
can be composed and converted into their borrowed or owned counterparts as
needed:

```rust
use std::collections::HashMap;

#[borrowme]
struct Word<'a> {
    text: &'a str,
}

#[borrowme]
struct Dictionary<'a> {
    words: HashMap<&'a str, Word<'a>>,
}

let dictionary = Dictionary {
    /* .. */
};

let owned_dictionary: OwnedDictionary = borrowme::to_owned(&dictionary);
let dictionary2: Dictionary<'_> = borrowme::borrow(&owned_dictionary);
```

<br>

[`Borrow`]: https://docs.rs/borrowme/latest/borrowme/trait.Borrow.html
[`BorrowMut`]: https://docs.rs/borrowme/latest/borrowme/trait.BorrowMut.html
[`ToOwned`]: https://docs.rs/borrowme/latest/borrowme/trait.ToOwned.html
[borrowme-derive]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html#why-isnt-this-a-derive
[borrowme]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html
[generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
[std-borrow-mut]: https://doc.rust-lang.org/std/borrow/trait.BorrowMut.html
[std-borrow]: https://doc.rust-lang.org/std/borrow/trait.Borrow.html
[std-to-owned]: https://doc.rust-lang.org/std/borrow/trait.ToOwned.html
