use borrowme::borrowme;

#[borrowme]
struct MissingLifetime {
    field: String,
}

#[borrowme]
struct MissingLifetimeGeneric<A, B, C> {
    field: (A, B, C),
}

fn main() {
}
