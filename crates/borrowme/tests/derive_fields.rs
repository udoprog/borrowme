#![allow(dead_code)]

use borrowme::borrowme;

#[borrowme]
struct DeriveStaticField<'a> {
    text: &'static str,
    lang: &'a str,
}
