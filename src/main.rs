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

use error::IndentexError;
use std::io::Write;
use transpile::TranspileOptions;

macro_rules! println_stderr(
    ($($arg:tt)*) => {
        writeln!(&mut ::std::io::stderr(), $($arg)*).unwrap();
    }
);

enum ReturnCode {
    Ok = 0,
    GenericError = 1,
    WalkError = 2,
    FileTypeError = 4,
    TranspilationError = 8,
}

fn main() {
    use clap::{App, Arg, ArgGroup};
    use std::path::Path;
    use std::process;
    use transpile::TranspileOptions;

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
        .arg(Arg::with_name("stdin")
            .help("Read Indentex source from standard input")
            .long("stdin")
            .requires("output"))
        .arg(Arg::with_name("stdout")
            .help("Write transpiled Indentex to standard output")
            .long("stdout"))
        .arg(Arg::with_name("out")
            .help("Write transpiled text to file")
            .long("out")
            .short("o")
            .takes_value(true)
            .value_name("FILE"))
        .group(ArgGroup::with_name("output")
            .args(&["out", "stdout"]))
        .group(ArgGroup::with_name("input")
            .args(&["path", "stdin"])
            .required(true))
        .get_matches();

    let use_single_file_mode = match m.value_of("path") {
        Some(p) => Path::new(p).is_file(),
        None => m.is_present("stdin"),
    };
    let use_directory_mode = match m.value_of("path") {
        Some(p) => Path::new(p).is_dir(),
        None => false,
    };
    let verbose = m.is_present("verbose");
    let stdout = m.is_present("stdout");
    let options = TranspileOptions {
        flatten_output: m.is_present("flatten-output"),
        prepend_do_not_edit_notice: ! m.is_present("disable-do-not-edit"),
    };

    let ret_val =
    if use_single_file_mode {
        // Single file mode
        match single_file_mode(m.value_of("path"), m.value_of("out"), stdout, &options) {
            Ok(_) => ReturnCode::Ok,
            Err(e) => {
                println_stderr!("Could not transpile: {}", e);
                ReturnCode::GenericError
            }
        }
    } else if use_directory_mode {
        // Directory mode
        if m.is_present("out") {
            println_stderr!("error: The argument --out/-o is not allowed for directories");
            ReturnCode::GenericError
        } else if m.is_present("stdout") {
            println_stderr!("error: The argument --stdout is not allowed for directories");
            ReturnCode::GenericError
        } else {
            match directory_mode(m.value_of("path").unwrap(), &options, verbose) {
                Ok(_) => ReturnCode::Ok,
                Err(_) => ReturnCode::GenericError,
            }
        }
    } else {
        println_stderr!("Error: path '{}' is neither a file nor a directory", m.value_of("path").unwrap());
        ReturnCode::FileTypeError
    };

    process::exit(ret_val as i32);
}

fn single_file_mode(in_path: Option<&str>, out_path: Option<&str>, stdout: bool,
        options: &TranspileOptions) -> Result<(), IndentexError> {

    use file_utils::{read, rename_indentex_file, write_to_file};
    use std::io::{self, Read};
    use std::path::Path;
    use transpile::transpile;

    let mut buffer = String::new();
    match in_path {
        Some(p) => {
            // Read from file
            buffer = read(&p)?;
        }
        None => {
            // Read from stdin
            io::stdin().read_to_string(&mut buffer)?;
        }
    }

    let transpiled_text = transpile(&buffer, options);

    match(stdout, out_path, in_path) {
        (true, _, _) => {
            // Write to stdout
            print!("{}", transpiled_text);
        }
        (_, Some(p), _) => {
            // Write to specified path
            write_to_file(Path::new(p), &transpiled_text)?;
        }
        (_, _, Some(p)) => {
            // Write to automatically determined path
            let path_out = rename_indentex_file(p)?;
            write_to_file(path_out, &transpiled_text)?;
        }
        _ => {
            // This should never happen because it's an already handled edge case
        }
    }

    Ok(())
}

fn directory_mode(path: &str, options: &TranspileOptions, verbose: bool)
        -> Result<(), IndentexError> {

    use file_utils::walk_indentex_files;
    use rayon::prelude::*;
    use std::path::PathBuf;
    use transpile::transpile_file;

    let batch: Vec<PathBuf> = walk_indentex_files(path)?;

    let successful: bool = batch.par_iter()
        .map(|p| match transpile_file(&p, &options) {
            Ok(_) => {
                if verbose {
                    println_stderr!("Transpiling file '{}'... ok", p.display());
                }
                true
            }
            Err(e) => {
                if verbose {
                    println_stderr!("Transpiling file '{}'... failed", p.display());
                }
                println_stderr!("Could not transpile '{}': {}", p.display(), e);
                false
            }
        })
        .all(|x| x);

    if successful {
        Ok(())
    } else {
        Err(IndentexError::TranspileError)
    }
}
