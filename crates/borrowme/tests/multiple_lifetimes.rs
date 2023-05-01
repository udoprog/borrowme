use borrowme::borrowme;

#[borrowme]
struct MultipleLifetimes<'a, 'b> {
    a: &'a str,
    b: &'b str,
}
