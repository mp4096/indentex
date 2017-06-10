#[macro_use]
extern crate clap;
extern crate globset;
extern crate ignore;
#[macro_use]
extern crate nom;
extern crate rayon;

#[macro_use]
mod helper_parsers;

mod error;
mod file_utils;
mod parsers;
mod transpile;

use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern fn indentex_transpile_file(path: *const c_char) -> i32 {
    use transpile::{transpile_file, TranspileOptions};

    let rust_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };

    return match transpile_file(rust_path.as_str(), &TranspileOptions {
        flatten_output: false,
        prepend_do_not_edit_notice: false,
    }) {
        Ok(_) => 0,
        Err(e) => {
            println!("Could not transpile '{}': {}", rust_path, e);
            1
        }
    };
}

#[no_mangle]
pub extern fn indentex_transpile_file_options(path: *const c_char, options: i32) -> i32 {
    use transpile::{transpile_file, TranspileOptions};

    let rust_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };

    return match transpile_file(rust_path.as_str(), &TranspileOptions {
        flatten_output: options & 1 != 0,
        prepend_do_not_edit_notice: options & 2 == 0,
    }) {
        Ok(_) => 0,
        Err(e) => {
            println!("Could not transpile '{}': {}", rust_path, e);
            1
        }
    }
}
