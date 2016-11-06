use nom;


pub enum Hashline {
    OpenEnv(Environment),
    PlainLine(String),
}

pub struct Environment {
    indent_depth: usize,
    name: String,
    opts: String,
    is_list_like: bool,
}

impl Environment {
    pub fn latex_begin(&self) -> String {
        format!(r"{:ind$}\begin{{{}}}{}",
                "",
                self.name,
                self.opts,
                ind = self.indent_depth)
    }

    pub fn latex_end(&self) -> String {
        format!(r"{:ind$}\end{{{}}}", "", self.name, ind = self.indent_depth)
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
named!(escaped_colon<u8>, preceded!(specific_byte!('\\' as u8), specific_byte!(':' as u8)));
named!(name_parser<u8>, alt!(escaped_colon | none_of_bytes_as_bytes!(b":([{ \t")));
named!(opts_parser<u8>, alt!(escaped_colon | none_of_bytes_as_bytes!(b":")));
named!(
    hashline_parser<Hashline>,
    do_parse!(
        ws: opt!(is_a!(" ")) >>
        tag!("# ") >>
        name: many1!(name_parser) >>
        opts: many0!(opts_parser) >>
        tag!(":") >>
        args: call!(nom::rest) >>
        (hashline_helper(ws.unwrap_or(&b""[..]), &name, &opts, &args))
    )
);
#[inline]
fn hashline_helper(ws: &[u8], name: &[u8], opts: &[u8], args: &[u8]) -> Hashline {
    use std::str::from_utf8;
    use self::Hashline::{PlainLine, OpenEnv};

    // It is ok to unwrap here, since we have checked for UTF-8 when we read the file
    let name_utf8 = from_utf8(name).unwrap().trim();
    let opts_utf8 = from_utf8(opts).unwrap().trim();
    let args_utf8 = from_utf8(args).unwrap().trim();

    if args_utf8.is_empty() {
        // If no args are given, it's an environment
        let env = Environment {
            indent_depth: ws.len(),
            name: name_utf8.to_string(),
            opts: opts_utf8.to_string(),
            is_list_like: list_env_parser(name).is_done(),
        };
        OpenEnv(env)
    } else {
        // If there are some args, it's a single-line command
        let ws_utf8 = from_utf8(ws).unwrap();
        PlainLine(format!(r"{}\{}{}{{{}}}", ws_utf8, name_utf8, opts_utf8, args_utf8))
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

    PlainLine(format!(r"{}\item {}", ws_utf8, item_utf8))
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

    match process_hashline(&line) {
        Some(r) => r,
        None => {
            if list_like_active {
                match process_itemline(&line) {
                    Some(r) => r,
                    None => PlainLine(line.as_ref().to_string()),
                }
            } else {
                PlainLine(line.as_ref().to_string())
            }
        }
    }
}
