#![allow(dead_code)]

use borrowme::borrowme;

// --- Struct shapes ---

/// Tuple struct with a single generic type param referenced by lifetime.
#[borrowme]
#[derive(Debug)]
struct Wrapper<'a, T>(&'a T);

/// Named-field struct with a single generic type param.
#[borrowme]
#[derive(Debug, PartialEq)]
struct Named<'a, T> {
    value: &'a T,
}

/// Mix: concrete borrowed str plus a generic type param.
#[borrowme]
#[derive(Debug, PartialEq)]
struct Mixed<'a, T> {
    text: &'a str,
    value: &'a T,
}

/// Multiple distinct generic type params.
#[borrowme]
#[derive(Debug, PartialEq)]
struct Multi<'a, A, B> {
    first: &'a A,
    second: &'a B,
}

/// Generic type param alongside multiple lifetimes.
#[borrowme]
#[derive(Debug, PartialEq)]
struct MultiLifetime<'a, 'b, T> {
    borrowed_str: &'a str,
    other_str: &'b str,
    value: &'a T,
}

/// Generic type param with an explicit custom owned name.
#[borrowme(name = WrapperBuf)]
#[derive(Debug)]
struct WrapperNamed<'a, T>(&'a T);

/// Generic type param field alongside a `#[copy]` primitive field.
#[borrowme]
#[derive(Debug, PartialEq)]
struct WithCopy<'a, T> {
    value: &'a T,
    count: u32,
}

/// Generic type param with an explicit `#[owned(…)]` override on the field
/// (the T param itself needs no Clone bound when the owned type is explicit).
#[derive(Clone)]
struct MyOwned(i32);

struct MyBorrowed<'a>(&'a i32);

impl borrowme::ToOwned for MyBorrowed<'_> {
    type Owned = MyOwned;
    fn to_owned(&self) -> MyOwned {
        MyOwned(*self.0)
    }
}

impl borrowme::Borrow for MyOwned {
    type Target<'a> = MyBorrowed<'a>;
    fn borrow(&self) -> MyBorrowed<'_> {
        MyBorrowed(&self.0)
    }
}

/// Struct that mixes a generic T with an explicit #[owned] override on another field.
#[borrowme]
#[derive(Debug)]
struct WithExplicitOwned<'a, T> {
    value: &'a T,
    #[owned(String)]
    label: &'a str,
}

// --- Enum shapes ---

/// Enum with a generic type param in one variant and a concrete type in another.
#[borrowme]
#[derive(Debug, PartialEq)]
enum Either<'a, T> {
    Left(&'a T),
    Right(&'a str),
    Empty,
}

/// Enum where multiple variants each use the generic type param.
#[borrowme]
#[derive(Debug, PartialEq)]
enum Repeated<'a, T> {
    First { value: &'a T },
    Second { value: &'a T, label: &'a str },
}

/// Enum with multiple generic type params.
#[borrowme]
#[derive(Debug, PartialEq)]
enum Choice<'a, A, B> {
    PickA(&'a A),
    PickB(&'a B),
}

// --- Attribute combinations ---

/// `#[owned_attr]` / `#[borrowed_attr]` work with generic type params.
#[borrowme]
#[owned_attr(derive(Debug))]
#[borrowed_attr(derive(Debug))]
struct Attributed<'a, T> {
    value: &'a T,
}

// =============================================================================
// Tests
// =============================================================================

/// Helper: to_owned then borrow roundtrip for Named<T>.
fn named_roundtrip<T>(val: T) -> OwnedNamed<T>
where
    T: Clone + std::fmt::Debug + PartialEq,
{
    let borrowed = Named { value: &val };
    let owned: OwnedNamed<T> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.value, val, "to_owned produced wrong value");

    let reborrowed: Named<'_, T> = borrowme::borrow(&owned);
    assert_eq!(*reborrowed.value, val, "borrow produced wrong value");
    owned
}

#[test]
fn test_named_roundtrip_string() {
    let owned = named_roundtrip(String::from("hello borrowme"));
    assert_eq!(owned.value, "hello borrowme");
}

#[test]
fn test_named_roundtrip_i32() {
    let owned = named_roundtrip(42i32);
    assert_eq!(owned.value, 42);
}

#[test]
fn test_named_roundtrip_vec() {
    let owned = named_roundtrip(vec![1u8, 2, 3]);
    assert_eq!(owned.value, vec![1, 2, 3]);
}

#[test]
fn test_named_roundtrip_tuple() {
    let owned = named_roundtrip((true, 99u64));
    assert_eq!(owned.value, (true, 99));
}

#[test]
fn test_wrapper_tuple_struct() {
    let val = String::from("tuple");
    let borrowed = Wrapper(&val);
    let owned: OwnedWrapper<String> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.0, val);

    let reborrowed: Wrapper<'_, String> = borrowme::borrow(&owned);
    assert_eq!(*reborrowed.0, val);
}

#[test]
fn test_mixed_struct() {
    let val = 99u64;
    let borrowed = Mixed {
        text: "hi",
        value: &val,
    };
    let owned: OwnedMixed<u64> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.text, "hi");
    assert_eq!(owned.value, 99u64);

    let reborrowed: Mixed<'_, u64> = borrowme::borrow(&owned);
    assert_eq!(reborrowed.text, "hi");
    assert_eq!(*reborrowed.value, 99u64);
}

#[test]
fn test_multi_type_params() {
    let a = String::from("alpha");
    let b = 7i64;
    let borrowed = Multi {
        first: &a,
        second: &b,
    };
    let owned: OwnedMulti<String, i64> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.first, a);
    assert_eq!(owned.second, b);

    let reborrowed: Multi<'_, String, i64> = borrowme::borrow(&owned);
    assert_eq!(*reborrowed.first, a);
    assert_eq!(*reborrowed.second, b);
}

#[test]
fn test_multi_lifetime() {
    let a = String::from("first");
    let b = String::from("second");
    let val = 123u32;
    let borrowed = MultiLifetime {
        borrowed_str: &a,
        other_str: &b,
        value: &val,
    };
    let owned: OwnedMultiLifetime<u32> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.borrowed_str, a);
    assert_eq!(owned.other_str, b);
    assert_eq!(owned.value, val);

    let reborrowed: MultiLifetime<'_, '_, u32> = borrowme::borrow(&owned);
    assert_eq!(reborrowed.borrowed_str, a.as_str());
    assert_eq!(reborrowed.other_str, b.as_str());
    assert_eq!(*reborrowed.value, val);
}

#[test]
fn test_with_copy_field() {
    let val = String::from("with copy");
    let borrowed = WithCopy { value: &val, count: 5 };
    let owned: OwnedWithCopy<String> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.value, val);
    assert_eq!(owned.count, 5);

    let reborrowed: WithCopy<'_, String> = borrowme::borrow(&owned);
    assert_eq!(*reborrowed.value, val);
    assert_eq!(reborrowed.count, 5);
}

#[test]
fn test_with_explicit_owned_field() {
    let val = String::from("val");
    let borrowed = WithExplicitOwned {
        value: &val,
        label: "my label",
    };
    let owned: OwnedWithExplicitOwned<String> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.value, val);
    assert_eq!(owned.label, "my label");
}

#[test]
fn test_enum_left_variant() {
    let val = String::from("left");
    let borrowed: Either<'_, String> = Either::Left(&val);
    let owned: OwnedEither<String> = borrowme::to_owned(&borrowed);
    assert_eq!(owned, OwnedEither::Left(val.clone()));

    let reborrowed: Either<'_, String> = borrowme::borrow(&owned);
    assert_eq!(reborrowed, Either::Left(&val));
}

#[test]
fn test_enum_right_variant() {
    let borrowed: Either<'_, String> = Either::Right("concrete");
    let owned: OwnedEither<String> = borrowme::to_owned(&borrowed);
    assert_eq!(owned, OwnedEither::Right(String::from("concrete")));

    let reborrowed: Either<'_, String> = borrowme::borrow(&owned);
    assert_eq!(reborrowed, Either::Right("concrete"));
}

#[test]
fn test_enum_empty_variant() {
    let borrowed: Either<'_, String> = Either::Empty;
    let owned: OwnedEither<String> = borrowme::to_owned(&borrowed);
    assert_eq!(owned, OwnedEither::Empty);
}

#[test]
fn test_enum_repeated_variants() {
    let val = 42i32;

    let first: Repeated<'_, i32> = Repeated::First { value: &val };
    let owned: OwnedRepeated<i32> = borrowme::to_owned(&first);
    assert_eq!(owned, OwnedRepeated::First { value: val });

    let second: Repeated<'_, i32> = Repeated::Second {
        value: &val,
        label: "x",
    };
    let owned2: OwnedRepeated<i32> = borrowme::to_owned(&second);
    assert_eq!(
        owned2,
        OwnedRepeated::Second {
            value: val,
            label: String::from("x"),
        }
    );
}

#[test]
fn test_enum_multiple_type_params() {
    let a = 1u8;
    let b = String::from("b");

    let pick_a: Choice<'_, u8, String> = Choice::PickA(&a);
    let owned_a: OwnedChoice<u8, String> = borrowme::to_owned(&pick_a);
    assert_eq!(owned_a, OwnedChoice::PickA(a));

    let pick_b: Choice<'_, u8, String> = Choice::PickB(&b);
    let owned_b: OwnedChoice<u8, String> = borrowme::to_owned(&pick_b);
    assert_eq!(owned_b, OwnedChoice::PickB(b));
}

#[test]
fn test_custom_name() {
    let val = 77i32;
    let borrowed = WrapperNamed(&val);
    let owned: WrapperBuf<i32> = borrowme::to_owned(&borrowed);
    assert_eq!(owned.0, val);

    let reborrowed: WrapperNamed<'_, i32> = borrowme::borrow(&owned);
    assert_eq!(*reborrowed.0, val);
}
