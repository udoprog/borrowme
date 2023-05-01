use borrowme::borrowme;

#[borrowme]
enum Enum<'a> {
    Variant {
        #[owned(String)]
        a: &'a str,
    },
    Unnamed(#[owned(String)] &'a str),
    Empty,
}
