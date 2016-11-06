extern crate ansi_term;
extern crate clap;
extern crate globset;
extern crate ignore;
#[macro_use]
extern crate nom;

// Import helper macros before `parsers`
#[macro_use]
mod helper_parsers;

mod error;
mod file_utils;
mod parsers;
mod transpile;


fn main() {
    use ansi_term::Colour::{Red, Green};
    use clap::{App, Arg};
    use file_utils::walk_indentex_files;
    use std::path::{Path, PathBuf};
    use std::process;
    use transpile::transpile_file;

    let m = App::new("indentex")
        .version("0.1.0")
        .author("Mikhail Pak <mikhail.pak@tum.de>")
        .about("Transpiler for an indentation-based LaTeX superset")
        .arg(Arg::with_name("path")
            .help("Path to a single indentex file or a directory (recursively transpile all \
                   indentex files)")
            .index(1)
            .required(true))
        .arg(Arg::with_name("verbose")
            .help("Show transpilation progress")
            .short("v")
            .long("verbose"))
        .get_matches();

    let path = Path::new(m.value_of("path").unwrap());
    let verbose = m.is_present("verbose");

    let mut ret_val: i32 = 0;

    let batch: Vec<PathBuf> = if path.is_file() {
        vec![path.to_path_buf()]
    } else if path.is_dir() {
        match walk_indentex_files(&path) {
            Ok(b) => b,
            Err(e) => {
                ret_val = 2;
                println!("{}", Red.bold().paint(format!("{}", e)));
                Vec::new()
            }
        }
    } else {
        ret_val = 4;
        println!("{}",
                 Red.bold().paint(format!("Error: path '{}' is neither a file nor a directory",
                                          path.display())));
        Vec::new()
    };

    for p in &batch {
        if verbose {
            print!("Transpiling file '{}'... ", p.display());
        }
        match transpile_file(&p) {
            Ok(_) => {
                if verbose {
                    println!("{}", Green.paint("ok"));
                }
            }
            Err(e) => {
                ret_val = 8;
                if verbose {
                    println!("{}", Red.paint("failed"));
                    print!("    ");
                }
                println!("{}",
                         Red.bold().paint(format!("Could not transpile '{}': {}", p.display(), e)));
            }
        }
    }

    process::exit(ret_val);
}
