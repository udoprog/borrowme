use borrowme::borrowme;

#[borrowme]
struct Struct<'a> {
    #[owned(ty = String)]
    a: &'a str,
}

#[borrowme]
struct Unnamed<'a>(#[owned(ty = String)] &'a str);

#[borrowme]
struct Empty;
