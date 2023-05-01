use borrowme::borrowme;

#[borrowme]
enum Enum<'a> {
    Variant {
        #[owned(ty = String)]
        a: &'a str,
    },
    Unnamed(#[owned(ty = String)] &'a str),
    Empty,
}
