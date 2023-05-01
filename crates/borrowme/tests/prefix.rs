use borrowme::borrowme;

#[borrowme(prefix = Prefix)]
struct Struct<'a> {
    #[owned(String)]
    a: &'a str,
}

#[borrowme(prefix = Prefix)]
struct Unnamed<'a>(#[owned(String)] &'a str);

#[borrowme(prefix = Prefix)]
struct Empty;
