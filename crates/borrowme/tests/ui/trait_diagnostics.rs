//! Test diagnostics emitted when a type doesn't implement a trait needed to be
//! part of a the owned struct.

use borrowme::borrowme;

struct MyType;
struct MyOwnedType;

#[borrowme]
#[derive(Clone, Debug)]
pub struct Word<'a> {
    #[borrowme(owned = MyOwnedType, to_owned_with = to_owned_my_type, borrow_with = borrow_my_type)]
    lang: &'a MyType,
}

fn to_owned_my_type(this: &MyType) -> MyOwnedType {
    MyOwnedType
}

fn borrow_my_type(this: &MyOwnedType) -> &MyType {
    &MyType
}

fn main() {
}
