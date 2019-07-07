#[derive(Debug, PartialEq)]
pub struct RawHashlineParseData {
    pub(super) indent_depth: usize,
    pub(super) name: String,
    pub(super) opts: String,
    pub(super) args: String,
    pub(super) comment: String,
}

#[derive(Debug, PartialEq)]
pub struct RawItemlineParseData {
    pub(super) indent_depth: usize,
    pub(super) item: String,
}

#[derive(Debug, PartialEq)]
pub enum Hashline {
    OpenEnv(Environment),
    PlainLine(String),
}

#[derive(Debug, PartialEq)]
pub struct Environment {
    indent_depth: usize,
    name: String,
    opts: String,
    comment: String,
    is_list_like: bool,
}

#[inline]
fn is_a_list_environment(input: &str) -> bool {
    fn parser(input: &str) -> nom::IResult<&str, &str> {
        use nom::branch::alt;
        use nom::bytes::complete::tag;

        alt((tag("itemize"), tag("enumerate"), tag("description")))(input)
    }
    parser(input.trim_start()).is_ok()
}

impl From<RawHashlineParseData> for Hashline {
    fn from(raw_hashline: RawHashlineParseData) -> Self {
        if raw_hashline.args.trim().is_empty() {
            // If no args are given, it's an environment
            let is_list_like = is_a_list_environment(raw_hashline.name.as_ref());
            Hashline::OpenEnv(Environment {
                indent_depth: raw_hashline.indent_depth,
                name: raw_hashline.name,
                opts: raw_hashline.opts,
                comment: raw_hashline.comment,
                is_list_like,
            })
        } else {
            // If there are some args, it's a single-line command
            Hashline::PlainLine(format!(
                r"{dummy:ind$}\{name}{opts}{{{args}}}{comment_sep}{comment}",
                dummy = "",
                ind = raw_hashline.indent_depth,
                name = raw_hashline.name,
                opts = raw_hashline.opts,
                args = raw_hashline.args,
                comment_sep = if raw_hashline.comment.is_empty() {
                    ""
                } else {
                    " "
                },
                comment = raw_hashline.comment.trim(),
            ))
        }
    }
}

impl From<RawItemlineParseData> for Hashline {
    fn from(raw_itemline: RawItemlineParseData) -> Self {
        Hashline::PlainLine(format!(
            r"{dummy:ind$}\item{item_sep}{content}",
            dummy = "",
            ind = raw_itemline.indent_depth,
            content = raw_itemline.item,
            item_sep = if raw_itemline.item.is_empty() {
                ""
            } else {
                " "
            },
        ))
    }
}

impl Environment {
    #[cfg(test)]
    pub fn new(
        indent_depth: usize,
        name: String,
        opts: String,
        comment: String,
        is_list_like: bool,
    ) -> Self {
        Self {
            indent_depth,
            name,
            opts,
            comment,
            is_list_like,
        }
    }

    pub fn latex_begin(&self) -> String {
        format!(
            r"{dummy:ind$}\begin{{{name}}}{opts}{comment_sep}{comment}",
            name = self.name,
            opts = self.opts,
            comment = self.comment,
            dummy = "",
            ind = self.indent_depth,
            comment_sep = if self.comment.is_empty() { "" } else { " " },
        )
    }

    pub fn latex_end(&self) -> String {
        format!(
            r"{dummy:ind$}\end{{{name}}}",
            name = self.name,
            dummy = "",
            ind = self.indent_depth,
        )
    }

    pub fn indent_depth(&self) -> usize {
        self.indent_depth
    }

    pub fn is_list_like(&self) -> bool {
        self.is_list_like
    }
}

// LCOV_EXCL_START
#[cfg(test)]
mod tests {
    #[test]
    fn list_environment_recognition() {
        use super::is_a_list_environment;

        assert_eq!(is_a_list_environment("itemize"), true);
        assert_eq!(is_a_list_environment("enumerate*"), true);
        assert_eq!(is_a_list_environment("  description  *"), true);
        assert_eq!(is_a_list_environment("    descriptionitemize"), true);
        assert_eq!(is_a_list_environment("item"), false);
        assert_eq!(is_a_list_environment("   itemiz"), false);
        assert_eq!(is_a_list_environment("   foobar"), false);
    }

    #[cfg(test)]
    mod raw_hashline_parser_data_into_hashline {
        use super::super::{Hashline, RawHashlineParseData};

        #[test]
        fn plain_lines() {
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 0,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    args: "bar".to_string(),
                    comment: "".to_string()
                }),
                Hashline::PlainLine("\\foo{bar}".to_string())
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 2,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    args: "bar".to_string(),
                    comment: "qux".to_string()
                }),
                Hashline::PlainLine("  \\foo{bar} qux".to_string())
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 4,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    args: "qux".to_string(),
                    comment: "".to_string()
                }),
                Hashline::PlainLine("    \\foobar{qux}".to_string())
            );
        }

        #[test]
        fn environments() {
            use super::super::Environment;

            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 0,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    args: "".to_string(),
                    comment: "".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 0,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    comment: "".to_string(),
                    is_list_like: false,
                })
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 2,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    args: "".to_string(),
                    comment: "bar".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 2,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    comment: "bar".to_string(),
                    is_list_like: false,
                })
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 4,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    args: "".to_string(),
                    comment: "qux".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 4,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    comment: "qux".to_string(),
                    is_list_like: false,
                })
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 0,
                    name: "itemize".to_string(),
                    opts: "bar".to_string(),
                    args: "".to_string(),
                    comment: "qux".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 0,
                    name: "itemize".to_string(),
                    opts: "bar".to_string(),
                    comment: "qux".to_string(),
                    is_list_like: true,
                })
            );
        }
    }

    #[test]
    fn raw_itemline_parser_data_into_hashline() {
        use super::{Hashline, RawItemlineParseData};

        assert_eq!(
            Hashline::from(RawItemlineParseData {
                indent_depth: 0,
                item: "".to_string()
            }),
            Hashline::PlainLine(r"\item".to_string())
        );
        assert_eq!(
            Hashline::from(RawItemlineParseData {
                indent_depth: 0,
                item: "".to_string()
            }),
            Hashline::PlainLine(r"\item".to_string())
        );
        assert_eq!(
            Hashline::from(RawItemlineParseData {
                indent_depth: 2,
                item: "".to_string()
            }),
            Hashline::PlainLine(r"  \item".to_string())
        );
        assert_eq!(
            Hashline::from(RawItemlineParseData {
                indent_depth: 0,
                item: "foo".to_string()
            }),
            Hashline::PlainLine(r"\item foo".to_string())
        );
        assert_eq!(
            Hashline::from(RawItemlineParseData {
                indent_depth: 3,
                item: "bar".to_string()
            }),
            Hashline::PlainLine(r"   \item bar".to_string())
        );
        assert_eq!(
            Hashline::from(RawItemlineParseData {
                indent_depth: 0,
                item: "**".to_string()
            }),
            Hashline::PlainLine(r"\item **".to_string())
        );
    }

    #[test]
    fn environment_methods() {
        use super::Environment;

        let env_1 = Environment {
            indent_depth: 0,
            name: "foo".to_string(),
            opts: "bar".to_string(),
            comment: "% baz".to_string(),
            is_list_like: true,
        };

        assert_eq!(env_1.latex_begin(), "\\begin{foo}bar % baz");
        assert_eq!(env_1.latex_end(), "\\end{foo}");
        assert_eq!(env_1.is_list_like(), true);
        assert_eq!(env_1.indent_depth(), 0);

        let env_2 = Environment {
            indent_depth: 2,
            name: "abc".to_string(),
            opts: "def".to_string(),
            comment: "".to_string(),
            is_list_like: false,
        };

        assert_eq!(env_2.latex_begin(), "  \\begin{abc}def");
        assert_eq!(env_2.latex_end(), "  \\end{abc}");
        assert_eq!(env_2.is_list_like(), false);
        assert_eq!(env_2.indent_depth(), 2);
    }
}
// LCOV_EXCL_STOP
