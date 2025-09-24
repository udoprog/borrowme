#![allow(dead_code)]

use borrowme::borrowme;

#[allow(non_camel_case_types)]
#[derive(Clone)]
struct bool;

impl borrowme::ToOwned for bool {
    type Owned = bool;

    fn to_owned(&self) -> Self::Owned {
        bool
    }
}

impl borrowme::Borrow for bool {
    type Target<'a> = &'a bool;

    fn borrow(&self) -> Self::Target<'_> {
        self
    }
}

impl AsRef<bool> for bool {
    fn as_ref(&self) -> &bool {
        self
    }
}

#[borrowme]
pub struct NoCopy<'a> {
    // Indicate that the field is not `Copy`, but it will have to implement
    // `borrowme` traits.
    #[borrowme(no_copy)]
    fake_bool_no_copy: &'a bool,
    // Rely on Clone and the field being a direct reference to the owned type.
    #[borrowme(std)]
    fake_bool_std: &'a bool,
    // Use custom methods and declarations.
    #[borrowme(owned = bool, borrow_with = AsRef::as_ref, to_owned_with = Clone::clone)]
    fake_bool_custom: &'a bool,
}
