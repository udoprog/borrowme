#![allow(dead_code)]

use borrowme::borrowme;

#[borrowme]
struct VecField<'a> {
    #[borrowme(borrow_with = Vec::as_slice)]
    strings: &'a [String],
}
