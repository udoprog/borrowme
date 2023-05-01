use borrowme::borrowme;

#[borrowme(prefix = Prefix)]
struct Struct<'a> {
    a: &'a str,
}

#[borrowme(prefix = Prefix)]
struct Unnamed<'a>(&'a str);

#[borrowme(prefix = Prefix)]
struct Empty;
