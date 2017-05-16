use nom;


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
        format!(r"{dummy:ind$}\begin{{{name}}}{opts}{comment_sep}{comment}",
                name = self.name,
                opts = self.opts,
                comment = self.comment,
                dummy = "",
                ind = self.indent_depth,
                comment_sep = if self.comment.is_empty() { "" } else { " " })
    }

    pub fn latex_end(&self) -> String {
        format!(r"{dummy:ind$}\end{{{name}}}",
                name = self.name,
                dummy = "",
                ind = self.indent_depth)
    }

    pub fn indent_depth(&self) -> usize {
        self.indent_depth
    }

    pub fn is_list_like(&self) -> bool {
        self.is_list_like
    }
}


// Hashline parsers
named!(
    list_env_parser<&[u8]>,
    ws!(alt!(tag!("itemize") | tag!("enumerate") | tag!("description")))
);
named!(escaped_colon<u8>, preceded!(specific_byte!(b'\\'), specific_byte!(b':')));
named!(escaped_percent<u8>, preceded!(specific_byte!(b'\\'), specific_byte!(b'%')));
named!(name_parser<u8>, alt!(escaped_colon | none_of_bytes_as_bytes!(b":%([{ \t")));
named!(opts_parser<u8>, alt!(escaped_colon | escaped_percent | none_of_bytes_as_bytes!(b":%")));
named!(args_parser<u8>, alt!(escaped_percent | none_of_bytes_as_bytes!(b"%")));
named!(
    hashline_parser<Hashline>,
    do_parse!(
        ws: opt!(is_a!(" ")) >>
        tag!("# ") >>
        name: many1!(name_parser) >>
        opts: many0!(opts_parser) >>
        tag!(":") >>
        args: many0!(args_parser) >>
        comment: call!(nom::rest) >>
        (hashline_helper(ws.unwrap_or(&b""[..]), &name, &opts, &args, comment))
    )
);
#[inline]
fn hashline_helper(ws: &[u8], name: &[u8], opts: &[u8], args: &[u8], comment: &[u8]) -> Hashline {
    use std::str::from_utf8;
    use self::Hashline::{PlainLine, OpenEnv};

    // It is ok to unwrap here, since we have checked for UTF-8 when we read the file
    let name_utf8 = from_utf8(name).unwrap().trim();
    let opts_utf8 = from_utf8(opts).unwrap().trim().replace("%", r"\%");
    let args_utf8 = from_utf8(args).unwrap().trim().replace("%", r"\%");
    let comment_utf8 = from_utf8(comment).unwrap().trim();

    if args_utf8.is_empty() {
        // If no args are given, it's an environment
        let env = Environment {
            indent_depth: ws.len(),
            name: name_utf8.to_string(),
            opts: opts_utf8.to_string(),
            comment: comment_utf8.to_string(),
            is_list_like: list_env_parser(name).is_done(),
        };
        OpenEnv(env)
    } else {
        // If there are some args, it's a single-line command
        let ws_utf8 = from_utf8(ws).unwrap();
        PlainLine(format!(r"{indent}\{name}{opts}{{{args}}}{comment_sep}{comment}",
                          indent = ws_utf8,
                          name = name_utf8,
                          opts = opts_utf8,
                          args = args_utf8,
                          comment_sep = if comment_utf8.is_empty() { "" } else { " " },
                          comment = comment_utf8))
    }
}

// Hashline processing
#[inline]
fn process_hashline<T: AsRef<str>>(line: T) -> Option<Hashline> {
    use nom::IResult::{Done, Error, Incomplete};

    match hashline_parser(line.as_ref().as_bytes()) {
        Done(_, r) => Some(r),
        Error(_) | Incomplete(_) => None,
    }
}


// Itemline parsers
named!(
    itemline_parser<Hashline>,
    do_parse!(
        ws: opt!(is_a!(" ")) >>
        tag!("*") >>
        item: call!(nom::rest) >>
        (itemline_helper(ws.unwrap_or(&b""[..]), item))
    )
);
#[inline]
fn itemline_helper(ws: &[u8], item: &[u8]) -> Hashline {
    use std::str::from_utf8;
    use self::Hashline::PlainLine;

    let ws_utf8 = from_utf8(ws).unwrap();
    let item_utf8 = from_utf8(item).unwrap().trim();

    PlainLine(format!(r"{indent}\item{item_sep}{content}",
                      indent = ws_utf8,
                      content = item_utf8,
                      item_sep = if item_utf8.is_empty() { "" } else { " " }))
}

// Itemline processing
#[inline]
fn process_itemline<T: AsRef<str>>(line: T) -> Option<Hashline> {
    use nom::IResult::{Done, Error, Incomplete};

    match itemline_parser(line.as_ref().as_bytes()) {
        Done(_, r) => Some(r),
        Error(_) | Incomplete(_) => None,
    }
}

// Fully process line
pub fn process_line<T>(line: T, list_like_active: bool) -> Hashline
    where T: AsRef<str>
{
    use self::Hashline::PlainLine;

    match (process_hashline(&line), list_like_active) {
        (Some(r), _) => r,
        (None, true) => process_itemline(&line).unwrap_or_else(|| PlainLine(line.as_ref().to_string())),
        (None, false) => PlainLine(line.as_ref().to_string()),
    }
}


#[cfg(test)]
mod tests {
    use nom::IResult::{Done, Error, Incomplete};
    use nom::{ErrorKind, Needed};

    macro_rules! nil { () => ("".as_bytes()); }
    macro_rules! ws_1 { () => (" ".as_bytes()); }
    macro_rules! ws_2 { () => ("  ".as_bytes()); }
    macro_rules! ws_4 { () => ("    ".as_bytes()); }

    macro_rules! foo { () => ("foo".as_bytes()); }
    macro_rules! bar { () => ("bar".as_bytes()); }
    macro_rules! qux { () => ("qux".as_bytes()); }
    macro_rules! itemize { () => ("itemize".as_bytes()); }

    #[test]
    fn hashline_helper_plain_lines() {
        use super::{Hashline, hashline_helper};

        assert_eq!(hashline_helper(nil!(), foo!(), nil!(), bar!(), nil!()),
                   Hashline::PlainLine("\\foo{bar}".to_string()));
        assert_eq!(hashline_helper(ws_2!(), foo!(), nil!(), bar!(), qux!()),
                   Hashline::PlainLine("  \\foo{bar} qux".to_string()));
        assert_eq!(hashline_helper(ws_4!(), foo!(), bar!(), qux!(), nil!()),
                   Hashline::PlainLine("    \\foobar{qux}".to_string()));
    }

    #[test]
    fn hashline_helper_environments() {
        use super::{Hashline, Environment, hashline_helper};

        let env_ref_1 = Environment {
            indent_depth: 0,
            name: "foo".to_string(),
            opts: "bar".to_string(),
            comment: "".to_string(),
            is_list_like: false,
        };
        assert_eq!(hashline_helper(nil!(), foo!(), bar!(), nil!(), nil!()),
                   Hashline::OpenEnv(env_ref_1));

        let env_ref_2 = Environment {
            indent_depth: 2,
            name: "foo".to_string(),
            opts: "".to_string(),
            comment: "bar".to_string(),
            is_list_like: false,
        };
        assert_eq!(hashline_helper(ws_2!(), foo!(), nil!(), nil!(), bar!()),
                   Hashline::OpenEnv(env_ref_2));

        let env_ref_3 = Environment {
            indent_depth: 4,
            name: "foo".to_string(),
            opts: "bar".to_string(),
            comment: "qux".to_string(),
            is_list_like: false,
        };
        assert_eq!(hashline_helper(ws_4!(), foo!(), bar!(), nil!(), qux!()),
                   Hashline::OpenEnv(env_ref_3));

        let env_ref_4 = Environment {
            indent_depth: 0,
            name: "itemize".to_string(),
            opts: "bar".to_string(),
            comment: "qux".to_string(),
            is_list_like: true,
        };
        assert_eq!(hashline_helper(nil!(), itemize!(), bar!(), nil!(), qux!()),
                   Hashline::OpenEnv(env_ref_4));
    }

    #[test]
    fn itemline_helper() {
        use super::{Hashline, itemline_helper};

        assert_eq!(itemline_helper(ws_2!(), foo!()),
                   Hashline::PlainLine("  \\item foo".to_string()));
        // Test that no whitespace is put after `\item` if no item is given
        assert_eq!(itemline_helper(ws_1!(), nil!()),
                   Hashline::PlainLine(" \\item".to_string()));
    }

    #[test]
    fn process_itemline() {
        use super::{Hashline, process_itemline};

        // Valid itemlines
        assert_eq!(process_itemline("*"),
                   Some(Hashline::PlainLine("\\item".to_string())));
        assert_eq!(process_itemline("*  "),
                   Some(Hashline::PlainLine("\\item".to_string())));
        assert_eq!(process_itemline("  *"),
                   Some(Hashline::PlainLine("  \\item".to_string())));
        assert_eq!(process_itemline("  *  "),
                   Some(Hashline::PlainLine("  \\item".to_string())));
        assert_eq!(process_itemline("* foo"),
                   Some(Hashline::PlainLine("\\item foo".to_string())));
        assert_eq!(process_itemline("  * bar"),
                   Some(Hashline::PlainLine("  \\item bar".to_string())));
        assert_eq!(process_itemline("****"),
                   Some(Hashline::PlainLine("\\item ***".to_string())));

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

        let a = b"itemize";
        let b = b"enumerate*";
        let c = b"    description  *";
        let d = b"item";
        let e = b"foobar";

        assert_eq!(list_env_parser(&a[..]), Done(&b""[..], &a[..]));
        assert_eq!(list_env_parser(&b[..]), Done(&b"*"[..], &b"enumerate"[..]));
        assert_eq!(list_env_parser(&c[..]), Done(&b"*"[..], &b"description"[..]));
        assert_eq!(list_env_parser(&d[..]), Incomplete(Needed::Size(7)));
        assert_eq!(list_env_parser(&e[..]), Error(error_position!(ErrorKind::Alt, &e[..])));
    }

    #[test]
    fn escaped_colon() {
        use super::escaped_colon;

        let a = br"\:";
        let c = b"ab";

        assert_eq!(escaped_colon(&a[..]), Done(&b""[..], ':' as u8));
        assert_eq!(escaped_colon(nil!()), Incomplete(Needed::Size(1)));
        assert_eq!(escaped_colon(&c[..]), Error(error_position!(ErrorKind::Char, &c[..])));
    }

    #[test]
    fn escaped_percent() {
        use super::escaped_percent;

        let a = br"\%";
        let c = b"ab";

        assert_eq!(escaped_percent(&a[..]), Done(&b""[..], '%' as u8));
        assert_eq!(escaped_percent(nil!()), Incomplete(Needed::Size(1)));
        assert_eq!(escaped_percent(&c[..]), Error(error_position!(ErrorKind::Char, &c[..])));
    }

    #[test]
    fn name_parser() {
        use super::name_parser;

        assert_eq!(name_parser(&br"abc"[..]), Done(&b"bc"[..], 'a' as u8));
        assert_eq!(name_parser(&br"\:abc"[..]), Done(&b"abc"[..], ':' as u8));
        assert_eq!(name_parser(&b""[..]), Incomplete(Needed::Size(1)));

        for e in vec![b":E", b"%E", b"(E", b"[E", b"{E", b" E", b"\tE"] {
            assert_eq!(name_parser(&e[..]), Error(error_position!(ErrorKind::Alt, &e[..])));
        }
    }

    #[test]
    fn opts_parser() {
        use super::opts_parser;

        assert_eq!(opts_parser(&br"abc"[..]), Done(&b"bc"[..], 'a' as u8));
        assert_eq!(opts_parser(&br"\:abc"[..]), Done(&b"abc"[..], ':' as u8));
        assert_eq!(opts_parser(&br"\%abc"[..]), Done(&b"abc"[..], '%' as u8));
        assert_eq!(opts_parser(&br"(abc"[..]), Done(&b"abc"[..], '(' as u8));
        assert_eq!(opts_parser(&br"[abc"[..]), Done(&b"abc"[..], '[' as u8));
        assert_eq!(opts_parser(&br" abc"[..]), Done(&b"abc"[..], ' ' as u8));
        assert_eq!(opts_parser(&b""[..]), Incomplete(Needed::Size(1)));

        for e in vec![b":E", b"%E"] {
            assert_eq!(opts_parser(&e[..]), Error(error_position!(ErrorKind::Alt, &e[..])));
        }
    }

    #[test]
    fn args_parser() {
        use super::args_parser;

        assert_eq!(args_parser(&br"abc"[..]), Done(&b"bc"[..], 'a' as u8));
        assert_eq!(args_parser(&br"\:abc"[..]), Done(&b":abc"[..], '\\' as u8));
        assert_eq!(args_parser(&br"\%abc"[..]), Done(&b"abc"[..], '%' as u8));
        assert_eq!(args_parser(&br"(abc"[..]), Done(&b"abc"[..], '(' as u8));
        assert_eq!(args_parser(&br"[abc"[..]), Done(&b"abc"[..], '[' as u8));
        assert_eq!(args_parser(&br" abc"[..]), Done(&b"abc"[..], ' ' as u8));
        assert_eq!(args_parser(&b""[..]), Incomplete(Needed::Size(1)));

        assert_eq!(args_parser(&b"%E"[..]), Error(error_position!(ErrorKind::Alt, &b"%E"[..])));
    }
}
