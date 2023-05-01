use borrowme::borrowme;

#[allow(non_camel_case_types)]
#[derive(Clone)]
struct bool;

#[borrowme]
pub struct Word<'a> {
    #[borrowme(std)]
    teineigo: &'a bool,
}
