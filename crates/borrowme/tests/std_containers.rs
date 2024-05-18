use borrowme::borrowme;

#[borrowme]
#[borrowme(std)]
#[derive(Clone)]
struct StdStruct<'a> {
    a: &'a String,
    #[borrowme(copy)]
    b: u32,
}

#[borrowme]
#[borrowme(std)]
#[derive(Clone)]
enum StdOnEnum<'a> {
    Variant {
        a: &'a String,
        #[borrowme(copy)]
        b: u32,
    },
}

#[borrowme]
#[derive(Clone)]
enum StdVariant<'a> {
    #[borrowme(std)]
    Variant {
        a: &'a String,
        #[borrowme(copy)]
        b: u32,
    },
}
