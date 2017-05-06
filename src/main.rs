extern crate ansi_term;
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
    use ansi_term::Colour::{Red, Green};
    use clap::{App, Arg};
    use file_utils::walk_indentex_files;
    use rayon::prelude::*;
    use std::cmp;
    use std::path::{Path, PathBuf};
    use std::process;
    use transpile::transpile_file;

    let m = App::new("indentex")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Transpiler for an indentation-based superset of LaTeX")
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
        .get_matches();

    let path = Path::new(m.value_of("path").unwrap());
    let verbose = m.is_present("verbose");
    let flatten_output = m.is_present("flatten-output");

    let mut ret_val = ReturnCode::Ok as i32;

    let batch: Vec<PathBuf> = if path.is_file() {
        vec![path.to_path_buf()]
    } else if path.is_dir() {
        match walk_indentex_files(&path) {
            Ok(b) => b,
            Err(e) => {
                ret_val = ReturnCode::WalkError as i32;
                println!("{}", Red.bold().paint(format!("{}", e)));
                Vec::new()
            }
        }
    } else {
        ret_val = ReturnCode::FileTypeError as i32;
        println!("{}",
                 Red.bold().paint(format!("Error: path '{}' is neither a file nor a directory",
                                          path.display())));
        Vec::new()
    };

    let ret_val_transpilation = batch.par_iter()
        .map(|p| match transpile_file(&p, flatten_output) {
            Ok(_) => {
                if verbose {
                    println!("Transpiling file '{}'... {}",
                             p.display(),
                             Green.paint("ok"));
                }
                ReturnCode::Ok
            }
            Err(e) => {
                if verbose {
                    println!("Transpiling file '{}'... {}",
                             p.display(),
                             Red.paint("failed"));
                }
                println!("{}",
                         Red.bold().paint(format!("Could not transpile '{}': {}", p.display(), e)));
                ReturnCode::TranspilationError
            }
        } as i32)
        .max()
        .unwrap_or(ReturnCode::Ok as i32);

    process::exit(cmp::max(ret_val, ret_val_transpilation));
}
