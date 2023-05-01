use borrowme::borrowme;

#[borrowme(prefix = Owned)]
struct DenyArguments<'a> {
    #[owned(ty = String)]
    a: &'a str,
}

fn main() {
}
