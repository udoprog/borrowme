//! Test diagnostics emitted when a type doesn't implement a trait needed to be
//! part of a the owned struct.

use borrowme::borrowme;

#[derive(Clone, Copy)]
struct MyType;

#[borrowme]
pub struct DuplicateAttribute<'a> {
    #[copy]
    #[no_copy]
    copy1: MyType,
    #[copy]
    #[copy]
    copy2: MyType,
    #[no_copy]
    #[no_copy]
    copy3: MyType,
    #[borrowme(std)]
    #[copy]
    std_and_copy: MyType,
    #[borrowme(mut)]
    #[borrowme(mut)]
    duplicate_mut: MyType,
    #[owned(String)]
    #[borrowme(owned = String)]
    owned1: &'a str,
    #[borrowme(with = path, borrow_with = path2)]
    path_conflict1: &'a str,
    #[borrowme(with = path, borrow_mut_with = path2)]
    path_conflict2: &'a str,
    #[borrowme(with = path, to_owned_with = path2)]
    path_conflict3: &'a str,
    string: &'a str,
}

fn main() {
}
