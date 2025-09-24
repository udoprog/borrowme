#![allow(dead_code)]

use borrowme::borrowme;

#[borrowme]
struct Struct<'a> {
    #[owned(String)]
    a: &'a str,
}

#[borrowme]
struct Unnamed<'a>(#[owned(String)] &'a str);

#[borrowme]
struct Empty;
