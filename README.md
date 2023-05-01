# borrowme

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/borrowme-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/borrowme)
[<img alt="crates.io" src="https://img.shields.io/crates/v/borrowme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/borrowme)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-borrowme-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/borrowme)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/borrowme/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/borrowme/actions?query=branch%3Amain)

The missing compositional borrowing for Rust.

This crate provides an attribute macro which helps you pair two types
through compositional borrowing and ownership conversion. Roughly this means
that you can convert a struct which has lifetimes into ones which does not
and such as between the `Word` and `OwnedWord` structs here:

```rust
#[derive(Debug, PartialEq, Eq)]
struct Word<'a> {
    text: &'a str,
    lang: Option<&'a str>,
    examples: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
struct OwnedWord {
    text: String,
    lang: Option<String>,
    examples: Vec<String>,
}
```

Writing and maintaining the `OwnedWord` variant is labour intensive and
error prone. Instead we can use the [`#[borrowme]`][borrowme] attribute
provided by this crate:

```rust
use borrowme::borrowme;

#[borrowme]
#[derive(Debug, PartialEq, Eq)]
struct Word<'a> {
    text: &'a str,
    lang: Option<&'a str>,
    examples: Vec<&'a str>,
}
```

See the [`#[borrowme]`][borrowme] attribute for detailed documentation on
how the attribute works.

```rust
let text = String::from("Hello");
let lang = Some(String::from("eng"));
let examples = vec![String::from("Hello World")];

let word = Word {
    text: "Hello World",
    lang: lang.as_deref(),
    examples: examples.iter().map(|s| s.as_str()).collect(),
};

let word2: OwnedWord = borrowme::to_owned(&word);
let word3: Word<'_> = borrowme::borrow(&word2);
assert_eq!(word3, word);
```

<br>

Rust comes with two sibling traits which both are responsible for converting
something to an owned and a borrowed variant: [`ToOwned`][std-to-owned] and
[`Borrow`][std-borrow].

These convert a type to a *borrowed* value to an owned one, let's think
about it from a broader perspective: How to we convert a type which *has
lifetimes*, to one which *does not*?

To this end this crate defines two similar traits: [`ToOwned`] and
[`Borrow`]. These traits serve a similar purpose to the traits in `std` but
are implemented differently. See their corresponding documentation for more
details.

[`ToOwned`]: https://docs.rs/borrowme/latest/borrowme/trait.ToOwned.html
[generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
[borrowme]: https://docs.rs/borrowme/latest/borrowme/attr.borrowme.html
[std-borrow]: std::borrow::Borrow
[std-to-owned]: std::borrow::ToOwned
