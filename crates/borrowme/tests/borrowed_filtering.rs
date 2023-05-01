use borrowme::borrowme;

#[borrowme]
#[borrowed(attr(derive(Default)))]
enum ImplementDefault<'a> {
    #[borrowed(attr(default))]
    Empty,
    Other(#[owned(ty = String)] &'a str),
}
