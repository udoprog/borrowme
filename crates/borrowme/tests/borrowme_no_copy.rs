use borrowme::borrowme;

#[borrowme]
#[allow(non_camel_case_types)]
struct bool;

#[borrowme]
pub struct NoCopy<'a> {
    #[borrowme(no_copy)]
    borrowme_no_copy: &'a bool,
}
