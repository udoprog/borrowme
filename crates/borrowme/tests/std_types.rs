use borrowme::borrowme;

use std::borrow::Cow;
use std::ffi::{CStr, OsStr};
use std::path::Path;

#[borrowme]
#[derive(Clone)]
struct CowStruct<'a> {
    owned_str_cow: Cow<'static, str>,
    borrowed_str_cow: Cow<'a, str>,
    borrowed_path_cow: Cow<'a, Path>,
    borrowed_bytes_cow: Cow<'a, [u8]>,
    borrowed_path: &'a Path,
    borrowed_os_str: &'a OsStr,
    borrowed_c_str: &'a CStr,
    #[borrowme(owned = Box<[u8]>, to_owned_with = Box::from)]
    boxed_bytes: &'a [u8],
}
