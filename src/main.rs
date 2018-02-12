#[macro_use]
extern crate clap;
extern crate globset;
extern crate ignore;
#[macro_use]
extern crate nom;
extern crate rayon;

// Import helper macros before `parsers`
#[macro_use]
mod helper_parsers;

mod error;
mod file_utils;
mod parsers;
mod transpile;

enum ReturnCode {
    Ok = 0,
    WalkError = 2,
    FileTypeError = 4,
    TranspilationError = 8,
}

fn main() {
    use clap::{App, Arg};
    use file_utils::walk_indentex_files;
    use rayon::prelude::*;
    use std::cmp;
    use std::path::{Path, PathBuf};
    use std::process;
    use transpile::{transpile_file, TranspileOptions};

    let m = App::new("indentex")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("path")
            .help("Path to a single indentex file or a directory (recursively transpile all \
                   indentex files)")
            .index(1)
            .required(true))
        .arg(Arg::with_name("verbose")
            .help("Show transpilation progress")
            .short("v")
            .long("verbose"))
        .arg(Arg::with_name("flatten-output")
            .help("Remove all indentation from the output")
            .long("flatten-output"))
        .arg(Arg::with_name("disable-do-not-edit")
            .help("Disable prepending the 'DO NOT EDIT' notice")
            .long("disable-do-not-edit"))
        .get_matches();

    let path = Path::new(m.value_of("path").unwrap());
    let verbose = m.is_present("verbose");
    let options = TranspileOptions {
        flatten_output: m.is_present("flatten-output"),
        prepend_do_not_edit_notice: ! m.is_present("disable-do-not-edit"),
    };

    let mut ret_val = ReturnCode::Ok as i32;

    let batch: Vec<PathBuf> = if path.is_file() {
        vec![path.to_path_buf()]
    } else if path.is_dir() {
        match walk_indentex_files(&path) {
            Ok(b) => b,
            Err(e) => {
                ret_val = ReturnCode::WalkError as i32;
                println!("{}", e);
                Vec::new()
            }
        }
    } else {
        ret_val = ReturnCode::FileTypeError as i32;
        println!("Error: path '{}' is neither a file nor a directory", path.display());
        Vec::new()
    };

    let ret_val_transpilation = batch.par_iter()
        .map(|p| match transpile_file(&p, &options) {
            Ok(_) => {
                if verbose {
                    println!("Transpiling file '{}'... ok", p.display());
                }
                ReturnCode::Ok
            }
            Err(e) => {
                if verbose {
                    println!("Transpiling file '{}'... failed", p.display());
                }
                println!("Could not transpile '{}': {}", p.display(), e);
                ReturnCode::TranspilationError
            }
        } as i32)
        .max()
        .unwrap_or(ReturnCode::Ok as i32);

    process::exit(cmp::max(ret_val, ret_val_transpilation));
}
