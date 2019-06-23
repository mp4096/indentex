#[inline]
fn count_left_indent<T: AsRef<str>>(line: T) -> Option<usize> {
    if line.as_ref().is_empty() {
        None
    } else {
        Some(line.as_ref().chars().count() - line.as_ref().trim_start().chars().count())
    }
}

pub fn scan_indents<T: AsRef<str>>(lines: &[T]) -> Vec<usize> {
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

#[inline]
pub fn trim_end_inplace(mut s: String) -> String {
    let len_to_truncate = s.trim_end().len();
    s.truncate(len_to_truncate);
    s
}

// LCOV_EXCL_START
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

    #[test]
    fn trim_end_inplace() {
        use super::trim_end_inplace;

        let foo = trim_end_inplace("  foo  ".to_string());
        assert_eq!(&foo, "  foo");
        assert_eq!(foo.len(), 5);

        let bar = trim_end_inplace("\t\tbar\t\t".to_string());
        assert_eq!(&bar, "\t\tbar");
        assert_eq!(bar.len(), 5);

        let qux = trim_end_inplace("\t qux \t".to_string());
        assert_eq!(&qux, "\t qux");
        assert_eq!(qux.len(), 5);

        let baz = trim_end_inplace("\t baz \n".to_string());
        assert_eq!(&baz, "\t baz");
        assert_eq!(baz.len(), 5);
    }
}
// LCOV_EXCL_STOP
