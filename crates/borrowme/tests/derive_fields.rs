use borrowme::borrowme;

#[borrowme]
#[derive(Debug, PartialEq, Eq)]
struct DeriveFields<'a> {
    text: &'a str,
    lang: Option<&'a str>,
    examples: Vec<&'a str>,
}
