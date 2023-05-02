use borrowme::borrowme;

#[borrowme(name = StructBuf)]
struct Struct<'a> {
    a: &'a str,
}

#[borrowme(name = UnnamedBuf)]
struct Unnamed<'a>(&'a str);

#[borrowme(name = EmptyBuf)]
struct Empty;
