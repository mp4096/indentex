mod error;
mod file_utils;

enum ReturnCode {
    Ok = 0,
    WalkError = 2,
    FileTypeError = 4,
    TranspilationError = 8,
}

pub fn transpile_file<T: AsRef<std::path::Path>>(
    path: T,
    options: &indentexlib::TranspileOptions,
) -> Result<(), crate::error::IndentexError> {
    use crate::file_utils::{read_and_trim_lines, rename_indentex_file, write_to_file};

    let lines = read_and_trim_lines(path.as_ref())?;
    let transpiled_text = indentexlib::transpile(lines, options);
    let path_out = rename_indentex_file(path)?;
    write_to_file(path_out, &transpiled_text)?;

    Ok(())
}

fn main() {
    use crate::file_utils::walk_indentex_files;
    use clap::{crate_authors, crate_description, crate_version, App, Arg};
    use indentexlib::TranspileOptions;
    use rayon::prelude::*;
    use std::path::{Path, PathBuf};

    let m = App::new("indentex")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("path")
                .help(
                    "Path to a single indentex file or a directory (recursively transpile all \
                     indentex files)",
                )
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Sets the level of verbosity (-v, -vv, -vvv)")
                .short("v")
                .multiple(true),
        )
        .arg(
            Arg::with_name("disable-do-not-edit")
                .help("Disable prepending the 'DO NOT EDIT' notice")
                .long("disable-do-not-edit"),
        )
        .get_matches();

    let log_level = match m.occurrences_of("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 | _ => log::LevelFilter::Trace,
    };
    env_logger::Builder::new().filter_level(log_level).init();

    let path = Path::new(m.value_of("path").unwrap());
    let options = TranspileOptions {
        prepend_do_not_edit_notice: !m.is_present("disable-do-not-edit"),
    };

    let mut ret_val = ReturnCode::Ok as i32;

    let batch: Vec<PathBuf> = if path.is_file() {
        vec![path.to_path_buf()]
    } else if path.is_dir() {
        match walk_indentex_files(&path) {
            Ok(b) => b,
            Err(e) => {
                ret_val = ReturnCode::WalkError as i32;
                log::error!("{}", e);
                Vec::new()
            }
        }
    } else {
        ret_val = ReturnCode::FileTypeError as i32;
        log::error!(
            "Error: path '{}' is neither a file nor a directory",
            path.display()
        );
        Vec::new()
    };

    let ret_val_transpilation = batch
        .par_iter()
        .map(|p| match transpile_file(&p, &options) {
            Ok(_) => {
                log::info!("Transpiling file '{}' ... ok", p.display());
                ReturnCode::Ok
            }
            Err(e) => {
                log::error!("Transpiling file '{}' ... failed: {}", p.display(), e);
                ReturnCode::TranspilationError
            }
        } as i32)
        .max()
        .unwrap_or(ReturnCode::Ok as i32);

    std::process::exit(std::cmp::max(ret_val, ret_val_transpilation));
}
