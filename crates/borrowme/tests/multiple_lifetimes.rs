use borrowme::borrowme;

#[borrowme]
struct MultipleLifetimes<'a, 'b> {
    #[owned(String)]
    a: &'a str,
    #[owned(String)]
    b: &'b str,
}
