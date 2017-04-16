use std::path::Path;
use std::vec::Vec;
use error::IndentexError;


const LINESEP: &'static str = "\n";
const LATEX_TO_INDENTEX_FACTOR: f64 = 1.5;


// Indentation processing
#[inline]
fn count_left_indent<T: AsRef<str>>(line: T) -> Option<usize> {
    if line.as_ref().is_empty() {
        None
    } else {
        Some(line.as_ref().chars().count() - line.as_ref().trim_left().chars().count())
    }
}

fn scan_indents<T: AsRef<str>>(lines: &[T]) -> Vec<usize> {
    let raw_indents = lines.iter().map(count_left_indent).collect::<Vec<_>>();

    let mut adjusted_indents: Vec<usize> = Vec::with_capacity(raw_indents.len() + 1);
    let mut last_indent: usize = 0;

    for current_indent in raw_indents.iter().rev() {
        adjusted_indents.push(match *current_indent {
            None => last_indent,
            Some(ind) => {
                last_indent = ind;
                ind
            }
        });
    }

    adjusted_indents.reverse();
    adjusted_indents.push(0);

    adjusted_indents
}


// Transpilation
fn transpile<T: AsRef<str>>(lines: &[T]) -> String {
    use parsers::Environment;
    use parsers::Hashline::{PlainLine, OpenEnv};
    use parsers::process_line;

    // The number of environments is not known beforehand
    let mut env_stack: Vec<Environment> = Vec::new();

    // Input size is the sum of all line lengths plus the number of lines (for lineseps)
    let input_size = lines.iter().fold(0, |sum, l| sum + l.as_ref().len()) + lines.len();
    // We do not know how much larger the transpiled LaTeX file will be, but we can guess...
    let indentex_size = (LATEX_TO_INDENTEX_FACTOR * (input_size as f64)).round() as usize;
    let mut transpiled = String::with_capacity(indentex_size);

    let adjusted_indents = scan_indents(lines);

    for (line_num, line) in lines.iter().enumerate() {
        let list_like_active = match env_stack.last() {
            None => false, // No environment is active at all
            Some(d) => d.is_list_like(),
        };

        let tl = match process_line(line.as_ref(), list_like_active) {
            PlainLine(l) => l,
            OpenEnv(e) => {
                let tag_begin = e.latex_begin();
                env_stack.push(e);
                tag_begin
            }
        };
        transpiled.push_str(&tl);
        transpiled.push_str(LINESEP);

        // Check if we are in an environment and close as many as needed
        while match env_stack.last() {
            None => false,
            Some(d) => d.indent_depth() >= adjusted_indents[line_num + 1],
        } {
            // `unwrap()` is safe here since we have already checked if the stack is empty
            let tag_end = env_stack.pop().unwrap().latex_end();
            transpiled.push_str(&tag_end);
            transpiled.push_str(LINESEP);
        }
    }

    transpiled
}

pub fn transpile_file<T: AsRef<Path>>(path: T) -> Result<(), IndentexError> {
    use file_utils::{read_and_trim_lines, rename_indentex_file, write_to_file};

    let lines = read_and_trim_lines(path.as_ref())?;
    let transpiled_text = transpile(&lines);
    let path_out = rename_indentex_file(path)?;
    write_to_file(path_out, &transpiled_text)?;

    Ok(())
}


#[cfg(test)]
mod tests {
    #[test]
    fn count_left_indent() {
        use super::count_left_indent;

        assert_eq!(count_left_indent(""), None);
        assert_eq!(count_left_indent("foo"), Some(0));
        assert_eq!(count_left_indent("  bar"), Some(2));
        // We assume that the input has no trailing whitespaces
        // This is not a bug (but not a nice behaviour either)
        assert_eq!(count_left_indent("   "), Some(3));
    }

    #[test]
    fn scan_indents() {
        use super::scan_indents;

        // Always add a zero at the end
        let a = [" a"];
        assert_eq!(scan_indents(&a), [1, 0]);
        assert_eq!(scan_indents(&a).capacity(), 2);
        // Indents are propagated backwards
        let b = ["  b", "b", "", "  b"];
        assert_eq!(scan_indents(&b), [2, 0, 2, 2, 0]);
        assert_eq!(scan_indents(&b).capacity(), 5);
        // We assume that the input has no trailing whitespaces
        // This is not a bug (but not a nice behaviour either)
        let c = ["", "   "];
        assert_eq!(scan_indents(&c), [3, 3, 0]);
        assert_eq!(scan_indents(&c).capacity(), 3);

        let d = ["d", " d", "", " d", "", "   d", "  d", "     d"];
        assert_eq!(scan_indents(&d), [0, 1, 1, 1, 3, 3, 2, 5, 0]);
        assert_eq!(scan_indents(&d).capacity(), 9);
    }
}
