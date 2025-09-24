#![allow(dead_code)]

use borrowme::borrowme;

#[borrowme]
#[borrowed_attr(derive(Default))]
enum ImplementDefault<'a> {
    #[borrowed_attr(default)]
    Empty,
    Other(#[owned(String)] &'a str),
}
