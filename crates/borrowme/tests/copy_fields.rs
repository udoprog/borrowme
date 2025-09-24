#![allow(dead_code)]

use borrowme::borrowme;

#[allow(non_camel_case_types)]
#[derive(Clone)]
struct bool;

#[derive(Clone, Copy)]
struct CopyType;

#[borrowme]
struct ExternalType<'a> {
    string: &'a str,
}

#[borrowme]
struct MixedFields<'a> {
    #[borrowme(std)]
    weird_type_heuristics: &'a bool,
    primitive_f32: f32,
    primitive_f64: f64,
    primitive_u8: u8,
    primitive_u16: u16,
    primitive_u32: u32,
    primitive_u64: u64,
    primitive_u128: u128,
    primitive_usize: usize,
    primitive_i8: i8,
    primitive_i16: i16,
    primitive_i32: i32,
    primitive_i64: i64,
    primitive_i128: i128,
    primitive_isize: isize,
    tuple_empty: (),
    tuple_copy: (u32, u32),
    array_copy: [u32; 8],
    #[copy]
    explicit_copy: CopyType,
    owned_string: String,
    owned_list: Vec<String>,
    external_type: Option<ExternalType<'a>>,
}
