use borrowme::borrowme;

#[borrowme]
struct LifetimeHint<'a, 'b> {
    #[owned(String)]
    a: &'a str,
    #[copy]
    b: &'b str,
}

fn main() {
}
