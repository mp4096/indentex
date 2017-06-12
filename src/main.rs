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
    use file_utils::{read, rename_indentex_file, walk_indentex_files, write_to_file};
    use std::io::{self, Read};
    use rayon::prelude::*;
    use std::cmp;
    use std::path::{Path, PathBuf};
    use std::process;
    use transpile::{transpile, transpile_file, TranspileOptions};

    let m = App::new("indentex")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Transpiler for an indentation-based superset of LaTeX")
        .arg(Arg::with_name("path")
            .help("Path to a single indentex file or a directory (recursively transpile all \
                   indentex files)")
            .index(1))
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
        .arg(Arg::with_name("stdout")
            .help("Print transpiled text to standard out")
            .long("stdout"))
        .arg(Arg::with_name("out")
            .help("Write transpiled text to file")
            .long("out")
            .short("o")
            .takes_value(true)
            .value_name("FILE"))
        .get_matches();

    let path = Path::new(m.value_of("path").unwrap_or(""));
    let verbose = m.is_present("verbose");
    let stdout = m.is_present("stdout");
    let options = TranspileOptions {
        flatten_output: m.is_present("flatten-output"),
        prepend_do_not_edit_notice: ! m.is_present("disable-do-not-edit"),
    };

    let mut ret_val = ReturnCode::Ok as i32;

    if ! m.is_present("path") ||  path.is_file() {
        // Single file mode
        let mut buffer = String::new();
        ret_val = if m.is_present("path") {
            // Read from file
            match read(&path) {
                Ok(b) => {
                    buffer = b;
                    ReturnCode::Ok as i32
                }
                Err(e) => {
                    println!("Could not read '{}': {}", path.display(), e);
                    ReturnCode::FileTypeError as i32
                }
            }
        } else {
            // Read from stdin
            match io::stdin().read_to_string(&mut buffer) {
                Ok(_) => {
                    ReturnCode::Ok as i32
                }
                Err(e) => {
                    println!("Could not read from stdin: {}", e);
                    ReturnCode::FileTypeError as i32
                }
            }
        };

        let transpiled_text = transpile(&buffer, &options);

        if stdout {
            print!("{}", transpiled_text);
        } else {
            match m.value_of("out") {
                Some(p) => {
                    // Write to specified path
                    write_to_file(Path::new(p), &transpiled_text).ok().unwrap(); // TODO
                }
                None => {
                    // Write to automatically determined path
                    let path_out = rename_indentex_file(path).ok().unwrap(); // TODO
                    write_to_file(path_out, &transpiled_text).ok().unwrap(); // TODO
                }
            }
        }
    } else if path.is_dir() {
        // Directory mode
        let batch: Vec<PathBuf> = match walk_indentex_files(&path) {
            Ok(b) => b,
            Err(e) => {
                ret_val = ReturnCode::WalkError as i32;
                println!("{}", e);
                Vec::new()
            }
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

        ret_val = cmp::max(ret_val, ret_val_transpilation);
    } else {
        println!("Error: path '{}' is neither a file nor a directory", path.display());
        ret_val = ReturnCode::FileTypeError as i32;
    }

    process::exit(ret_val);
}
