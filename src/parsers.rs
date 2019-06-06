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

impl Environment {
    pub fn latex_begin(&self) -> String {
        format!(
            r"{dummy:ind$}\begin{{{name}}}{opts}{comment_sep}{comment}",
            name = self.name,
            opts = self.opts,
            comment = self.comment,
            dummy = "",
            ind = self.indent_depth,
            comment_sep = if self.comment.is_empty() { "" } else { " " }
        )
    }

    pub fn latex_end(&self) -> String {
        format!(
            r"{dummy:ind$}\end{{{name}}}",
            name = self.name,
            dummy = "",
            ind = self.indent_depth
        )
    }

    pub fn indent_depth(&self) -> usize {
        self.indent_depth
    }

    pub fn is_list_like(&self) -> bool {
        self.is_list_like
    }
}

#[inline]
fn list_env_parser(input: &str) -> nom::IResult<&str, ()> {
    use nom::branch::alt;
    use nom::bytes::complete::{is_a, tag};
    use nom::combinator::opt;

    let (input, _) = opt(is_a(" "))(input)?;
    let (input, _) = alt((tag("itemize"), tag("enumerate"), tag("description")))(input)?;
    Ok((input, ()))
}

#[inline]
fn escaped_colon(input: &str) -> nom::IResult<&str, &str> {
    use nom::bytes::complete::tag;
    use nom::sequence::preceded;

    preceded(tag("\\"), tag(":"))(input)
}

#[inline]
fn name_parser(input: &str) -> nom::IResult<&str, &str> {
    use nom::branch::alt;
    use nom::bytes::complete::{is_not, tag};

    alt((escaped_colon, tag("\\"), is_not("\\:%([{ \t")))(input)
}

#[inline]
fn opts_parser(input: &str) -> nom::IResult<&str, &str> {
    use nom::branch::alt;
    use nom::bytes::complete::{is_not, tag};

    alt((escaped_colon, tag("\\%"), tag("\\"), is_not("\\:%")))(input)
}

#[inline]
fn args_parser(input: &str) -> nom::IResult<&str, &str> {
    use nom::branch::alt;
    use nom::bytes::complete::{is_not, tag};

    alt((tag("\\%"), tag("\\"), is_not("\\%")))(input)
}

fn hashline_parser(input: &str) -> nom::IResult<&str, Hashline> {
    use nom::bytes::complete::{is_a, tag};
    use nom::combinator::{opt, rest};
    use nom::multi::{fold_many0, fold_many1};

    let (input, res) = opt(is_a(" "))(input)?;
    let ws = res.unwrap_or("");
    let (input, _) = tag("# ")(input)?;
    let (input, name) = fold_many1(
        name_parser,
        String::with_capacity(input.len()),
        |mut acc: String, item| {
            acc.push_str(item);
            acc
        },
    )(input)?;
    let (input, opts) = fold_many0(
        opts_parser,
        String::with_capacity(input.len()),
        |mut acc: String, item| {
            acc.push_str(item);
            acc
        },
    )(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, args) = fold_many0(
        args_parser,
        String::with_capacity(input.len()),
        |mut acc: String, item| {
            acc.push_str(item);
            acc
        },
    )(input)?;
    let (input, comment) = rest(input)?;
    Ok((input, hashline_helper(&ws, &name, &opts, &args, comment)))
}

#[inline]
fn hashline_helper(ws: &str, name: &str, opts: &str, args: &str, comment: &str) -> Hashline {
    use self::Hashline::{OpenEnv, PlainLine};

    // It is ok to unwrap here, since we have checked for UTF-8 when we read the file
    let name_trimmed = name.trim();
    let opts_trimmed = opts.trim();
    let args_trimmed = args.trim();
    let comment_trimmed = comment.trim();

    if args_trimmed.is_empty() {
        // If no args are given, it's an environment
        let env = Environment {
            indent_depth: ws.len(),
            name: name_trimmed.to_string(),
            opts: opts_trimmed.to_string(),
            comment: comment_trimmed.to_string(),
            is_list_like: list_env_parser(name).is_ok(),
        };
        OpenEnv(env)
    } else {
        // If there are some args, it's a single-line command
        let ws_trimmed = ws;
        PlainLine(format!(
            r"{indent}\{name}{opts}{{{args}}}{comment_sep}{comment}",
            indent = ws_trimmed,
            name = name_trimmed,
            opts = opts_trimmed,
            args = args_trimmed,
            comment_sep = if comment_trimmed.is_empty() { "" } else { " " },
            comment = comment_trimmed
        ))
    }
}

// Hashline processing
#[inline]
fn process_hashline<T: AsRef<str>>(line: T) -> Option<Hashline> {
    match hashline_parser(line.as_ref()) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

// Itemline parsers
fn itemline_parser(input: &str) -> nom::IResult<&str, Hashline> {
    use self::Hashline::PlainLine;

    use nom::bytes::complete::{is_a, tag};
    use nom::combinator::{opt, rest};

    let (input, indent) = opt(is_a(" "))(input)?;
    let (input, _) = tag("*")(input)?;
    let (input, item) = rest(input)?;

    Ok((
        input,
        PlainLine(format!(
            r"{indent}\item{item_sep}{content}",
            indent = indent.unwrap_or(""),
            content = item.trim(),
            item_sep = if item.trim().is_empty() { "" } else { " " }
        )),
    ))
}

// Itemline processing
#[inline]
fn process_itemline<T: AsRef<str>>(line: T) -> Option<Hashline> {
    match itemline_parser(line.as_ref()) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

// Fully process line
pub fn process_line<T>(line: T, list_like_active: bool) -> Hashline
where
    T: AsRef<str>,
{
    use self::Hashline::PlainLine;

    match (process_hashline(&line), list_like_active) {
        (Some(r), _) => r,
        (None, true) => {
            process_itemline(&line).unwrap_or_else(|| PlainLine(line.as_ref().to_string()))
        }
        (None, false) => PlainLine(line.as_ref().to_string()),
    }
}

// LCOV_EXCL_START
#[cfg(test)]
mod tests {
    #[test]
    fn hashline_helper_plain_lines() {
        use super::{hashline_helper, Hashline};

        assert_eq!(
            hashline_helper("", "foo", "", "bar", ""),
            Hashline::PlainLine("\\foo{bar}".to_string())
        );
        assert_eq!(
            hashline_helper("  ", "foo", "", "bar", "qux"),
            Hashline::PlainLine("  \\foo{bar} qux".to_string())
        );
        assert_eq!(
            hashline_helper("    ", "foo", "bar", "qux", ""),
            Hashline::PlainLine("    \\foobar{qux}".to_string())
        );
    }

    #[test]
    fn hashline_helper_environments() {
        use super::{hashline_helper, Environment, Hashline};

        let env_ref_1 = Environment {
            indent_depth: 0,
            name: "foo".to_string(),
            opts: "bar".to_string(),
            comment: "".to_string(),
            is_list_like: false,
        };
        assert_eq!(
            hashline_helper("", "foo", "bar", "", ""),
            Hashline::OpenEnv(env_ref_1)
        );

        let env_ref_2 = Environment {
            indent_depth: 2,
            name: "foo".to_string(),
            opts: "".to_string(),
            comment: "bar".to_string(),
            is_list_like: false,
        };
        assert_eq!(
            hashline_helper("  ", "foo", "", "", "bar"),
            Hashline::OpenEnv(env_ref_2)
        );

        let env_ref_3 = Environment {
            indent_depth: 4,
            name: "foo".to_string(),
            opts: "bar".to_string(),
            comment: "qux".to_string(),
            is_list_like: false,
        };
        assert_eq!(
            hashline_helper("    ", "foo", "bar", "", "qux"),
            Hashline::OpenEnv(env_ref_3)
        );

        let env_ref_4 = Environment {
            indent_depth: 0,
            name: "itemize".to_string(),
            opts: "bar".to_string(),
            comment: "qux".to_string(),
            is_list_like: true,
        };
        assert_eq!(
            hashline_helper("", "itemize", "bar", "", "qux"),
            Hashline::OpenEnv(env_ref_4)
        );
    }

    #[test]
    fn process_itemline() {
        use super::{process_itemline, Hashline};

        // Valid itemlines
        assert_eq!(
            process_itemline("*"),
            Some(Hashline::PlainLine("\\item".to_string()))
        );
        assert_eq!(
            process_itemline("*  "),
            Some(Hashline::PlainLine("\\item".to_string()))
        );
        assert_eq!(
            process_itemline("  *"),
            Some(Hashline::PlainLine("  \\item".to_string()))
        );
        assert_eq!(
            process_itemline("  *  "),
            Some(Hashline::PlainLine("  \\item".to_string()))
        );
        assert_eq!(
            process_itemline("* foo"),
            Some(Hashline::PlainLine("\\item foo".to_string()))
        );
        assert_eq!(
            process_itemline("  * bar"),
            Some(Hashline::PlainLine("  \\item bar".to_string()))
        );
        assert_eq!(
            process_itemline("****"),
            Some(Hashline::PlainLine("\\item ***".to_string()))
        );

        // Not an itemline
        assert_eq!(process_itemline("  baz"), None);
        assert_eq!(process_itemline("qux *"), None);
        assert_eq!(process_itemline("  abc * def"), None);
        assert_eq!(process_itemline("  \\*  "), None);
        assert_eq!(process_itemline("\\*  "), None);
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

    #[test]
    fn list_env_parser() {
        use super::list_env_parser;
        use nom::error::ErrorKind::Tag;
        use nom::Err::Error;

        assert_eq!(list_env_parser("itemize"), Ok(("", ())));
        assert_eq!(list_env_parser("enumerate*"), Ok(("*", ())));
        assert_eq!(list_env_parser("    description  *"), Ok(("  *", ())));
        assert_eq!(list_env_parser("item"), Err(Error(("item", Tag))));
        assert_eq!(list_env_parser("   foobar"), Err(Error(("foobar", Tag))));
    }

    #[test]
    fn escaped_colon() {
        use super::escaped_colon;
        use nom::error::ErrorKind::Tag;
        use nom::Err::Error;

        assert_eq!(escaped_colon(r"\:"), Ok(("", ":")));
        assert_eq!(escaped_colon(""), Err(Error(("", Tag))));
        assert_eq!(escaped_colon("ab"), Err(Error(("ab", Tag))));
    }

    #[test]
    fn name_parser() {
        use super::name_parser;
        use nom::error::ErrorKind::IsNot;
        use nom::Err::Error;

        assert_eq!(name_parser("abc"), Ok(("", "abc")));
        assert_eq!(name_parser(r"abc\:"), Ok((r"\:", "abc")));
        assert_eq!(name_parser(r"\:abc"), Ok(("abc", ":")));
        assert_eq!(name_parser(" "), Err(Error((" ", IsNot))));
        assert_eq!(name_parser(""), Err(Error(("", IsNot))));

        for e in vec![":E", "%E", "(E", "[E", "{E", " E", "\tE"] {
            assert_eq!(name_parser(e), Err(Error((e, IsNot))));
        }
    }

    #[test]
    fn opts_parser() {
        use super::opts_parser;
        use nom::error::ErrorKind::IsNot;
        use nom::Err::Error;

        assert_eq!(opts_parser(r"abc"), Ok(("", "abc")));
        assert_eq!(opts_parser(r"\:abc"), Ok(("abc", ":")));
        assert_eq!(opts_parser(r"\%abc"), Ok(("abc", r"\%")));
        assert_eq!(opts_parser(r"(abc"), Ok(("", "(abc")));
        assert_eq!(opts_parser(r"[abc"), Ok(("", "[abc")));
        assert_eq!(opts_parser(r" abc"), Ok(("", " abc")));
        assert_eq!(opts_parser(""), Err(Error(("", IsNot))));

        for e in vec![":E", "%E"] {
            assert_eq!(opts_parser(e), Err(Error((e, IsNot))));
        }
    }

    #[test]
    fn args_parser() {
        use super::args_parser;
        use nom::error::ErrorKind::IsNot;
        use nom::Err::Error;

        assert_eq!(args_parser(r"abc"), Ok(("", "abc")));
        assert_eq!(args_parser(r"\:abc"), Ok((":abc", r"\")));
        assert_eq!(args_parser(r"\%abc"), Ok(("abc", r"\%")));
        assert_eq!(args_parser(r"(abc"), Ok(("", "(abc")));
        assert_eq!(args_parser(r"[abc"), Ok(("", "[abc")));
        assert_eq!(args_parser(r" abc"), Ok(("", " abc")));
        assert_eq!(args_parser(""), Err(Error(("", IsNot))));
        assert_eq!(args_parser("%E"), Err(Error(("%E", IsNot))));
    }
}
// LCOV_EXCL_STOP
