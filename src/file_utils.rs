use std::path::{Path, PathBuf};
use std::vec::Vec;
use error::IndentexError;


const INDENTEX_GLOB: &'static str = "*.inden.tex";


pub fn walk_indentex_files<T: AsRef<Path>>(rootdir: T) -> Result<Vec<PathBuf>, IndentexError> {
    use ignore::WalkBuilder;
    use ignore::types::TypesBuilder;

    // Create a type matcher
    let mut tb = TypesBuilder::new();
    tb.add("indentex", INDENTEX_GLOB).unwrap();
    tb.select("indentex");
    let matcher = tb.build().unwrap();

    // Walk the path
    let mut files: Vec<PathBuf> = Vec::new();
    for res in WalkBuilder::new(rootdir.as_ref()).types(matcher).build() {
        let item = res?;
        if item.file_type().unwrap().is_file() {
            files.push(item.path().to_path_buf());
        }
    }

    Ok(files)
}

#[inline]
fn is_indentex_file<T: AsRef<Path>>(filepath: T) -> bool {
    use globset::Glob;

    let glob = Glob::new(INDENTEX_GLOB).unwrap().compile_matcher();
    glob.is_match(filepath.as_ref())
}


/// Rename an `*.inden.tex` file into `*_inden.tex`
pub fn rename_indentex_file<T: AsRef<Path>>(old_path: T) -> Result<PathBuf, IndentexError> {
    if !is_indentex_file(old_path.as_ref()) {
        return Err(IndentexError::InvalidExtension);
    }

    let mut new_pathbuf = old_path.as_ref().to_path_buf();
    // Strip both extensions, first `.tex` and then `.inden`
    // We assume that the file `old_path` is a valid indentex file,
    // i.e. it has the
    new_pathbuf.set_extension("");
    new_pathbuf.set_extension("");
    // Get the full filename (i.e. with all dots etc. if there are any)
    let mut new_name = new_pathbuf.file_name().unwrap().to_os_string();
    new_name.push("_indentex.tex");
    new_pathbuf.pop();
    new_pathbuf.push(new_name);

    Ok(new_pathbuf)
}

/// Read a file line by line, right-trim lines and _copy_ them into a vec of strings
pub fn read_and_trim_lines<T: AsRef<Path>>(path: T) -> Result<Vec<String>, IndentexError> {
    use std::fs::File;
    use std::io::{BufReader, BufRead};

    if !is_indentex_file(path.as_ref()) {
        return Err(IndentexError::InvalidExtension);
    }

    let file = File::open(path.as_ref())?;
    let buf = BufReader::new(file);

    buf.lines().map(|r| Ok(r?.trim_right().to_string())).collect()
}

pub fn write_to_file<T, U>(path: T, data: U) -> Result<(), IndentexError>
    where T: AsRef<Path>,
          U: AsRef<str>
{

    use std::fs::File;
    use std::io::{BufWriter, Write};

    let file = File::create(path.as_ref())?;
    let mut buf = BufWriter::new(file);
    buf.write_all(data.as_ref().as_bytes())?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn is_indentex_file() {
        use super::is_indentex_file;

        assert!(!is_indentex_file(Path::new("foo")));
        assert!(!is_indentex_file(Path::new("foo.tex")));
        assert!(!is_indentex_file(Path::new("foo_inden.tex")));

        assert!(is_indentex_file(Path::new("foo.inden.tex")));
        assert!(is_indentex_file(Path::new("foo.bar.inden.tex")));
    }

    #[test]
    fn rename_indentex_file() {
        use super::rename_indentex_file;

        assert_eq!(rename_indentex_file(Path::new("./foo.inden.tex")).unwrap(),
                   PathBuf::from("./foo_indentex.tex"));
        assert_eq!(rename_indentex_file(Path::new("./foo.bar.inden.tex")).unwrap(),
                   PathBuf::from("./foo.bar_indentex.tex"));
        assert_eq!(rename_indentex_file(Path::new("./.foo.bar.inden.tex")).unwrap(),
                   PathBuf::from("./.foo.bar_indentex.tex"));
        assert_eq!(rename_indentex_file(Path::new("foo.inden.tex")).unwrap(),
                   PathBuf::from("foo_indentex.tex"));
        assert!(rename_indentex_file(Path::new("foo.bar.tex")).is_err())
    }
}
