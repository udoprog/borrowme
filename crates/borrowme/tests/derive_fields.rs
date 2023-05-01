use borrowme::borrowme;

#[borrowme]
struct DeriveFields<'a> {
    text: &'a str,
    lang: Option<&'a str>,
    examples: Vec<&'a str>,
}

#[borrowme]
struct DeriveStaticField<'a> {
    text: &'static str,
    lang: &'a str,
}
