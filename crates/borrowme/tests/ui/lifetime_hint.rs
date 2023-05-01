use borrowme::borrowme;

#[borrowme]
struct LifetimeHint<'a, 'b> {
    #[owned(ty = String)]
    a: &'a str,
    b: &'b str,
}

fn main() {
}
