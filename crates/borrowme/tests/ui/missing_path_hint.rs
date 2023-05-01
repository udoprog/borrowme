use borrowme::borrowme;

#[borrowme]
struct VecField<'a> {
    #[borrowme(borrow_with = std::slice::as_slice)]
    strings: &'a [String],
}
