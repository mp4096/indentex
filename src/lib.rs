#[macro_use]
extern crate clap;
extern crate globset;
extern crate ignore;
extern crate libc;
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
use libc::c_char;
use libc::c_void;
use libc::size_t;
use transpile::TranspileOptions;

enum TranspileFlags {
    FlattenOutput    = 0b01,
    DisableDoNotEdit = 0b10,
}


fn transpile_options_from_flags(flags: i32) -> TranspileOptions {
    TranspileOptions {
        flatten_output: flags & TranspileFlags::FlattenOutput as i32 != 0,
        prepend_do_not_edit_notice: flags & TranspileFlags::DisableDoNotEdit as i32 == 0,
    }
}


#[no_mangle]
pub extern fn indentex_transpile_file(path: *const c_char) -> i32 {
    return indentex_transpile_file_flags(path, 0);
}


#[no_mangle]
pub extern fn indentex_transpile_file_flags(path: *const c_char, flags: i32) -> i32 {
    use transpile::transpile_file;

    let rust_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };

    return match transpile_file(rust_path.as_str(), &transpile_options_from_flags(flags)) {
        Ok(_) => 0,
        Err(e) => {
            println!("Could not transpile '{}': {}", rust_path, e);
            1
        }
    }
}


#[no_mangle]
pub extern fn indentex_transpile(input: *const c_void, input_len: size_t,
                                output: *mut c_void, output_len: size_t) -> i32 {
    return indentex_transpile_flags(input, input_len, output, output_len, 0);
}


#[no_mangle]
pub extern fn indentex_transpile_flags(input: *const c_void, input_len: size_t,
                                output: *mut c_void, output_len: size_t, flags: i32) -> i32 {
    use std::slice;
    use transpile::transpile;
    use std::str;
    use std::ptr::copy_nonoverlapping;

    let slic = unsafe { slice::from_raw_parts(input as *const u8, input_len) };
    let buf = str::from_utf8(slic).unwrap();
    let lines: Vec<String> = buf.lines().map(|r| r.trim_right().to_string()).collect();
    let transpiled_text = transpile(&lines, &transpile_options_from_flags(flags));
    let transpiled_bytes = transpiled_text.into_bytes();
    if transpiled_bytes.len() <= output_len {
        unsafe {
            copy_nonoverlapping(transpiled_bytes.as_ptr(), output as *mut u8, transpiled_bytes.len());
        }
        return 0;
    } else {
        return 1;
    }
}
