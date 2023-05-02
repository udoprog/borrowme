use borrowme::borrowme;

#[borrowme]
struct Inner<'a> {
    text: &'static str,
    lang: &'a mut String,
}

#[borrowme]
struct BorrowMutStruct<'a> {
    text: &'a str,
    #[borrowme(mut)]
    inner: Inner<'a>,
}

#[borrowme]
enum BorrowMutEnum<'a> {
    Variant {
        text: &'a str,
        #[borrowme(mut)]
        inner: Inner<'a>,
    },
    Variant2 {
        text: &'a str,
        #[borrowme(mut)]
        inner: Inner<'a>,
    },
}
