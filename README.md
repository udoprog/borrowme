# borrowme

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/borrowme-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/borrowme)
[<img alt="crates.io" src="https://img.shields.io/crates/v/borrowme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/borrowme)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-borrowme-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/borrowme)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/borrowme/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/borrowme/actions?query=branch%3Amain)

The missing compositional borrowing for Rust.

Rust comes with two sibling traits which that can convert from owned to
borrowed and vice versa: [`ToOwned`][std-to-owned] and
[`Borrow`][std-borrow].

These can convert most simple types such as `&str` to and from `String`. But
lets think of this in a broader perspective. How to we convert a type that
*has lifetimes*, to one which *does not*? This crate defines its own
[`ToOwned`] and [`Borrow`] traits which serve a similar purpose to the ones
in `std` but are implemented so that they can do this generically.

To help us implement these traits the [`#[borrowme]`][borrowme] attribute
macro is provided ([see this section][borrowme-derive] for why it's not a
derive macro).

```rust
#[borrowme]
#[derive(Debug, Clone)]
struct Word<'a> {
    text: &'a str,
    lang: Option<&'a str>,
    examples: Vec<&'a str>,
}

```

With this we get the following additional structs and trait implementations:

```rust
#[derive(Debug, Clone)]
struct OwnedWord {
    text: String,
    lang: Option<String>,
    examples: Vec<String>,
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

<br>

[`Borrow`]: https://docs.rs/borrowme/latest/borrowme/trait.Borrow.html
[`ToOwned`]: https://docs.rs/borrowme/latest/borrowme/trait.ToOwned.html
[borrowme-derive]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html#why-isnt-this-a-derive
[borrowme]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html
[generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
[std-borrow]: std::borrow::Borrow
[std-to-owned]: std::borrow::ToOwned
