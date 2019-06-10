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
fn name_chunk_parser(input: &str) -> nom::IResult<&str, &str> {
    use nom::branch::alt;
    use nom::bytes::complete::is_not;

    alt((escaped_colon, is_not("\\:%([{ \t")))(input)
}

#[inline]
fn name_parser(input: &str) -> nom::IResult<&str, String> {
    nom::multi::fold_many1(
        name_chunk_parser,
        String::with_capacity(input.len()),
        |mut acc: String, item| {
            acc.push_str(item);
            acc
        },
    )(input)
}

#[inline]
fn opts_chunk_parser(input: &str) -> nom::IResult<&str, &str> {
    use nom::branch::alt;
    use nom::bytes::complete::{is_not, tag};

    alt((escaped_colon, tag("\\%"), tag("\\"), is_not("\\:%")))(input)
}

#[inline]
fn opts_parser(input: &str) -> nom::IResult<&str, String> {
    nom::multi::fold_many0(
        opts_chunk_parser,
        String::with_capacity(input.len()),
        |mut acc: String, item| {
            acc.push_str(item);
            acc
        },
    )(input)
}

#[inline]
fn args_chunk_parser(input: &str) -> nom::IResult<&str, &str> {
    use nom::branch::alt;
    use nom::bytes::complete::{is_not, tag};

    alt((tag("\\%"), tag("\\"), is_not("\\%")))(input)
}

#[inline]
fn args_parser(input: &str) -> nom::IResult<&str, String> {
    nom::multi::fold_many0(
        args_chunk_parser,
        String::with_capacity(input.len()),
        |mut acc: String, item| {
            acc.push_str(item);
            acc
        },
    )(input)
}

fn hashline_parser(input: &str) -> nom::IResult<&str, Hashline> {
    use nom::bytes::complete::{is_a, tag};
    use nom::combinator::{opt, rest};

    let (input, res) = opt(is_a(" "))(input)?;
    let ws = res.unwrap_or("");
    let (input, _) = tag("# ")(input)?;
    let (input, name) = name_parser(input)?;
    let (input, opts) = opts_parser(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, args) = args_parser(input)?;
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
        PlainLine(format!(
            r"{indent}\{name}{opts}{{{args}}}{comment_sep}{comment}",
            indent = ws,
            name = name_trimmed,
            opts = opts_trimmed,
            args = args_trimmed,
            comment_sep = if comment_trimmed.is_empty() { "" } else { " " },
            comment = comment_trimmed,
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
            item_sep = if item.trim().is_empty() { "" } else { " " },
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
    #[cfg(test)]
    mod helper_parser_tests {
        #[test]
        fn escaped_colon() {
            use super::super::escaped_colon;
            use nom::error::ErrorKind::Tag;
            use nom::Err::Error;

            assert_eq!(escaped_colon(r"\:"), Ok(("", ":")));
            assert_eq!(escaped_colon(r"\"), Err(Error((r"", Tag))));
            assert_eq!(escaped_colon(r":\"), Err(Error((r":\", Tag))));
            assert_eq!(escaped_colon(r"\\"), Err(Error((r"\", Tag))));
            assert_eq!(escaped_colon(r"\a"), Err(Error((r"a", Tag))));
            assert_eq!(escaped_colon(r"\;"), Err(Error((r";", Tag))));
            assert_eq!(escaped_colon(""), Err(Error(("", Tag))));
            assert_eq!(escaped_colon("ab"), Err(Error(("ab", Tag))));
        }
    }

    macro_rules! name_parser_valid_input_examples {
        () => {
            vec![
                "abc",
                "áàê",
                "äüß",
                "абв",
                "!!",
                "@@",
                "##",
                "&&",
                "==",
                "--",
                "__",
                "//",
                ";;",
                ",,",
                "..",
                "**",
                "||",
                "??",
                "\"\"",
                "''",
                "section",
                "section*",
                "equation*",
            ]
        };
    }

    macro_rules! name_parser_valid_input_with_escaped_colons_examples {
        () => {
            vec![
                r"\:abc\:",
                r"äü\:ß\:",
                r"а\:б\:в",
                r"!!\:",
                r"@\:@",
                r"\:#\:#",
                r"&\:&",
                r"\:==\:",
                r"-\:-\:",
                r"_\:_",
                r"\:/\:/",
                r";\:\:;",
                r",\:,\:",
                r".\:.\:",
                r"*\:*",
                r"\:|\:|",
                r"\:?\:?\:",
                "\"\\:\\:\"",
                r"\:'\:'",
                r"\:sec\:tion",
                r"section\:*\:",
                r"\:equation\:*",
                r"\:\:\:\:",
            ]
        };
    }

    #[cfg(test)]
    mod name_chunk_parser_spec {
        use super::super::name_chunk_parser;

        #[test]
        fn should_take_whole_input() {
            for input in name_parser_valid_input_examples!() {
                assert_eq!(name_chunk_parser(input), Ok(("", input)));
            }
        }

        #[test]
        fn should_take_only_the_escaped_colon_at_the_beginning() {
            for valid_input in name_parser_valid_input_examples!() {
                let input = r"\:".to_string() + valid_input;
                assert_eq!(name_chunk_parser(&input), Ok((valid_input, ":")));
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            use nom::error::ErrorKind::IsNot;
            use nom::Err::Error;

            for terminator in " :%([{\t\\".chars() {
                for valid_input in name_parser_valid_input_examples!() {
                    let input = terminator.to_string() + valid_input;
                    assert_eq!(
                        name_chunk_parser(&input),
                        Err(Error((input.as_ref(), IsNot)))
                    );
                }
            }
        }

        #[test]
        fn should_stop_at_a_terminator_after_taking_as_much_as_possible() {
            for terminator in " :%([{\t\\".chars() {
                for valid_input in name_parser_valid_input_examples!() {
                    let expected_rest = terminator.to_string() + valid_input;
                    let input_with_terminator = valid_input.to_string() + expected_rest.as_ref();
                    assert_eq!(
                        name_chunk_parser(&input_with_terminator),
                        Ok((expected_rest.as_ref(), valid_input))
                    );
                }
            }
        }

        #[test]
        fn should_stop_at_an_escaped_colon_after_taking_as_much_as_possible() {
            for valid_input in name_parser_valid_input_examples!() {
                let expected_rest = r"\:".to_string() + valid_input;
                let input_with_escaped_colon = valid_input.to_string() + expected_rest.as_ref();
                assert_eq!(
                    name_chunk_parser(&input_with_escaped_colon),
                    Ok((expected_rest.as_ref(), valid_input))
                );
            }
        }

        #[test]
        fn realistic_examples() {
            assert_eq!(
                name_chunk_parser("equation: foo"),
                Ok((": foo", "equation"))
            );
            assert_eq!(
                name_chunk_parser("equation : foo"),
                Ok((" : foo", "equation"))
            );
            assert_eq!(
                name_chunk_parser("equation* : foo"),
                Ok((" : foo", "equation*"))
            );
            assert_eq!(
                name_chunk_parser("equation[bar]: foo"),
                Ok(("[bar]: foo", "equation"))
            );
            assert_eq!(
                name_chunk_parser(r"foo\:bar: qux"),
                Ok((r"\:bar: qux", "foo"))
            );
        }
    }

    #[cfg(test)]
    mod name_parser_spec {
        use super::super::name_parser;

        #[test]
        fn should_take_whole_input() {
            for input in name_parser_valid_input_examples!() {
                assert_eq!(name_parser(input), Ok(("", input.to_string())));
            }
        }

        #[test]
        fn should_take_whole_input_and_replace_escaped_colons() {
            for input in name_parser_valid_input_with_escaped_colons_examples!() {
                assert_eq!(
                    name_parser(input),
                    Ok(("", input.replace(r"\:", ":")))
                );
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            use nom::error::ErrorKind::Many1;
            use nom::Err::Error;

            for terminator in " :%([{\t\\".chars() {
                for valid_input in name_parser_valid_input_examples!() {
                    let input = terminator.to_string() + valid_input;
                    assert_eq!(name_parser(&input), Err(Error((input.as_ref(), Many1))));
                }
            }
        }

        #[test]
        fn should_stop_at_a_terminator_after_taking_as_much_as_possible() {
            for terminator in " :%([{\t\\".chars() {
                for valid_input in name_parser_valid_input_with_escaped_colons_examples!() {
                    let expected_rest = terminator.to_string() + valid_input;
                    let input_with_terminator = valid_input.to_string() + expected_rest.as_ref();
                    assert_eq!(
                        name_parser(&input_with_terminator),
                        Ok((expected_rest.as_ref(), valid_input.replace(r"\:", ":")))
                    );
                }
            }
        }

        #[test]
        fn realistic_examples() {
            assert_eq!(
                name_parser("equation: foo"),
                Ok((": foo", "equation".to_string()))
            );
            assert_eq!(
                name_parser("equation : foo"),
                Ok((" : foo", "equation".to_string()))
            );
            assert_eq!(
                name_parser("equation* : foo"),
                Ok((" : foo", "equation*".to_string()))
            );
            assert_eq!(
                name_parser("equation[bar]: foo"),
                Ok(("[bar]: foo", "equation".to_string()))
            );
            assert_eq!(
                name_parser(r"foo\:bar: qux"),
                Ok((": qux", "foo:bar".to_string()))
            );
        }
    }

    macro_rules! opts_parser_valid_input_examples {
        () => {
            vec![
                "abc",
                "áàê",
                "äüß",
                "абв",
                "!!",
                "@@",
                "##",
                "&&",
                "==",
                "-;-",
                "__",
                "//",
                ";;",
                ",,",
                "..",
                "()",
                ")(",
                "[]",
                "][",
                "{}",
                "}{",
                "<>",
                "><",
                "**",
                "||",
                "??",
                "\"\"",
                "''",
                "      ",
                "section",
                "section*",
                "equation*",
            ]
        };
    }

    macro_rules! opts_parser_valid_input_with_escaped_chars_examples {
        () => {
            vec![
                "abc",
                "áàê",
                "äüß",
                "абв",
                r"\!!",
                r"@@\:",
                r"\\##",
                r"&\%&",
                r"=\\\\\=",
                r"-\:\;-",
                r"\%__",
                r"//\%",
                r";;\%",
                ",,",
                "..",
                "()",
                ")(",
                "[]",
                "][",
                "{}",
                "}{",
                "<>",
                "><",
                "**",
                "||",
                "??",
                "\"\"",
                "''",
                "      ",
                "section",
                "section*",
                "equation*",
            ]
        };
    }

    #[cfg(test)]
    mod opts_chunk_parser_spec {
        use super::super::opts_chunk_parser;

        #[test]
        fn should_take_whole_input() {
            for input in opts_parser_valid_input_examples!() {
                assert_eq!(opts_chunk_parser(input), Ok(("", input)));
            }
        }

        #[test]
        fn should_take_only_the_escaped_percent_at_the_beginning() {
            for valid_input in opts_parser_valid_input_examples!() {
                let input = r"\%".to_string() + valid_input;
                assert_eq!(opts_chunk_parser(&input), Ok((valid_input, r"\%")));
            }
        }

        #[test]
        fn should_take_only_the_escaped_colon_at_the_beginning() {
            for valid_input in opts_parser_valid_input_examples!() {
                let input = r"\:".to_string() + valid_input;
                assert_eq!(opts_chunk_parser(&input), Ok((valid_input, ":")));
            }
        }

        #[test]
        fn should_take_only_the_backslash_at_the_beginning() {
            for valid_input in opts_parser_valid_input_examples!() {
                let input = r"\".to_string() + valid_input;
                assert_eq!(opts_chunk_parser(&input), Ok((valid_input, r"\")));
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            use nom::error::ErrorKind::IsNot;
            use nom::Err::Error;

            for terminator in "%:".chars() {
                for valid_input in opts_parser_valid_input_examples!() {
                    let input = terminator.to_string() + valid_input;
                    assert_eq!(
                        opts_chunk_parser(&input),
                        Err(Error((input.as_ref(), IsNot)))
                    );
                }
            }
        }

        #[test]
        fn should_stop_at_a_terminator_or_escaped_char_after_taking_as_much_as_possible() {
            for stop_sequence in vec![r"\", r"\%", r"\:", ":"] {
                for valid_input in opts_parser_valid_input_examples!() {
                    let expected_rest = stop_sequence.to_string() + valid_input;
                    let input_with_stop_sequence = valid_input.to_string() + expected_rest.as_ref();
                    assert_eq!(
                        opts_chunk_parser(&input_with_stop_sequence),
                        Ok((expected_rest.as_ref(), valid_input))
                    );
                }
            }
        }

        #[test]
        fn prefer_escaped_percent_to_backslash() {
            assert_eq!(opts_chunk_parser(r"\\%"), Ok((r"\%", r"\")));
            assert_eq!(opts_chunk_parser(r"\%\"), Ok((r"\", r"\%")));
        }

        #[test]
        fn prefer_escaped_colon_to_backslash() {
            assert_eq!(opts_chunk_parser(r"\\:"), Ok((r"\:", r"\")));
            assert_eq!(opts_chunk_parser(r"\:\"), Ok((r"\", ":")));
        }

        #[test]
        fn realistic_examples() {
            assert_eq!(
                opts_chunk_parser("equation: foo"),
                Ok((": foo", "equation"))
            );
            assert_eq!(
                opts_chunk_parser(r"\: foo"),
                Ok((" foo", ":"))
            );
            assert_eq!(
                opts_chunk_parser(r"\% equation : foo"),
                Ok((" equation : foo", r"\%"))
            );
            assert_eq!(
                opts_chunk_parser("equation* : foo"),
                Ok((": foo", "equation* "))
            );
            assert_eq!(
                opts_chunk_parser(r"$\mathcal{H}_2$"),
                Ok((r"\mathcal{H}_2$", "$"))
            );
            assert_eq!(
                opts_chunk_parser(r"\textbf{\texttt{$\frac{1}{2}$}}"),
                Ok((r"textbf{\texttt{$\frac{1}{2}$}}", r"\"))
            );
        }
    }

    #[cfg(test)]
    mod opts_parser_tests {
        use super::super::opts_parser;

        #[test]
        fn should_take_whole_input() {
            for valid_input in opts_parser_valid_input_examples!() {
                assert_eq!(opts_parser(valid_input), Ok(("", valid_input.to_string())));
            }
        }

        #[test]
        fn should_take_whole_input_and_replace_escaped_colons() {
            for valid_input in opts_parser_valid_input_with_escaped_chars_examples!() {
                assert_eq!(opts_parser(valid_input), Ok(("", valid_input.replace(r"\:", ":"))));
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            for terminator in ":%".chars() {
             for valid_input in opts_parser_valid_input_with_escaped_chars_examples!() {
                let input = terminator.to_string() + valid_input;
                assert_eq!(opts_parser(&input), Ok((input.as_ref(), "".to_string())));
            }}
        }

        #[test]
        fn should_stop_at_a_terminator_after_taking_as_much_as_possible() {
            for terminator in ":%".chars() {
            for valid_input in opts_parser_valid_input_with_escaped_chars_examples!() {
                let expected_rest = terminator.to_string() + valid_input.as_ref();
                let input_with_terminator = valid_input.to_string() + expected_rest.as_ref();
                assert_eq!(
                    opts_parser(&input_with_terminator),
                    Ok((expected_rest.as_ref(), valid_input.replace(r"\:", ":")))
                );
            }}
        }

        #[test]
        fn opts_parser_() {
            assert_eq!(opts_parser("abc"), Ok(("", "abc".to_string())));
            assert_eq!(opts_parser(r"abc\:"), Ok(("", "abc:".to_string())));
            assert_eq!(opts_parser(r"\:abc"), Ok(("", ":abc".to_string())));
            assert_eq!(opts_parser("abc def"), Ok(("", "abc def".to_string())));
            assert_eq!(opts_parser(r"abc\:def"), Ok(("", "abc:def".to_string())));
            assert_eq!(opts_parser(r"abc\:\\"), Ok(("", r"abc:\\".to_string())));
            assert_eq!(opts_parser(r"\"), Ok(("", r"\".to_string())));
            assert_eq!(opts_parser(r"\\"), Ok(("", r"\\".to_string())));
            assert_eq!(opts_parser(r"\\\"), Ok(("", r"\\\".to_string())));
            assert_eq!(opts_parser(r"\\:\"), Ok(("", r"\:\".to_string())));
            assert_eq!(opts_parser(" "), Ok(("", " ".to_string())));
            assert_eq!(opts_parser(""), Ok(("", "".to_string())));
            assert_eq!(
                opts_parser("equation: foo"),
                Ok((r": foo", "equation".to_string()))
            );
            assert_eq!(
                opts_parser("equation : foo"),
                Ok((r": foo", "equation ".to_string()))
            );
            assert_eq!(
                opts_parser("equation [bar]: foo"),
                Ok((r": foo", "equation [bar]".to_string()))
            );
            assert_eq!(
                opts_parser("equation {bar}: foo"),
                Ok((r": foo", "equation {bar}".to_string()))
            );
            assert_eq!(
                opts_parser(r"equation {\bar\%}: foo"),
                Ok((r": foo", r"equation {\bar\%}".to_string()))
            );
            assert_eq!(
                opts_parser(r"equation {bar\: qux}: foo"),
                Ok((r": foo", r"equation {bar: qux}".to_string()))
            );

            for e in vec![":E", "%E"] {
                assert_eq!(opts_parser(e), Ok((e, "".to_string())));
            }
        }
    }

    macro_rules! args_parser_valid_input_examples {
        () => {
            vec![
                "abc",
                "áàê",
                "äüß",
                "абв",
                "!!",
                "@@:",
                "##",
                "&&",
                "==",
                "-:;-",
                "__",
                "//",
                ";;",
                ",,",
                "..",
                "()",
                ")(",
                "[]",
                "][",
                "{}",
                "}{",
                "<>",
                "><",
                "**",
                "||",
                "??",
                "\"\"",
                "''",
                "      ",
                "section",
                "section*",
                "equation*",
            ]
        };
    }

    macro_rules! args_parser_valid_input_with_escaped_chars_examples {
        () => {
            vec![
                "abc",
                "áàê",
                "äüß",
                "абв",
                r"\!!",
                r"@@\:",
                r"\\##",
                r"&\%&",
                r"=\\\\\=",
                r"-\:\;-",
                r"\%__",
                r"//\%",
                r";;\%",
                ",,",
                "..",
                "()",
                ")(",
                "[]",
                "][",
                "{}",
                "}{",
                "<>",
                "><",
                "**",
                "||",
                "??",
                "\"\"",
                "''",
                "      ",
                "section",
                "section*",
                "equation*",
            ]
        };
    }

    #[cfg(test)]
    mod args_chunk_parser_spec {
        use super::super::args_chunk_parser;

        #[test]
        fn should_take_whole_input() {
            for input in args_parser_valid_input_examples!() {
                assert_eq!(args_chunk_parser(input), Ok(("", input)));
            }
        }

        #[test]
        fn should_take_only_the_escaped_percent_at_the_beginning() {
            for valid_input in args_parser_valid_input_examples!() {
                let input = r"\%".to_string() + valid_input;
                assert_eq!(args_chunk_parser(&input), Ok((valid_input, r"\%")));
            }
        }

        #[test]
        fn should_take_only_the_backslash_at_the_beginning() {
            for valid_input in args_parser_valid_input_examples!() {
                let input = r"\".to_string() + valid_input;
                assert_eq!(args_chunk_parser(&input), Ok((valid_input, r"\")));
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            use nom::error::ErrorKind::IsNot;
            use nom::Err::Error;

            for valid_input in args_parser_valid_input_examples!() {
                let input = "%".to_string() + valid_input;
                assert_eq!(
                    args_chunk_parser(&input),
                    Err(Error((input.as_ref(), IsNot)))
                );
            }
        }

        #[test]
        fn should_stop_at_a_terminator_or_escaped_char_after_taking_as_much_as_possible() {
            for stop_sequence in vec!["%", r"\", r"\%"] {
                for valid_input in args_parser_valid_input_examples!() {
                    let expected_rest = stop_sequence.to_string() + valid_input;
                    let input_with_stop_sequence = valid_input.to_string() + expected_rest.as_ref();
                    assert_eq!(
                        args_chunk_parser(&input_with_stop_sequence),
                        Ok((expected_rest.as_ref(), valid_input))
                    );
                }
            }
        }

        #[test]
        fn prefer_escaped_percent_to_backslash() {
            assert_eq!(args_chunk_parser(r"\\%"), Ok((r"\%", r"\")));
            assert_eq!(args_chunk_parser(r"\%\"), Ok((r"\", r"\%")));
        }

        #[test]
        fn realistic_examples() {
            assert_eq!(
                args_chunk_parser("equation: foo"),
                Ok(("", "equation: foo"))
            );
            assert_eq!(
                args_chunk_parser(r"\% equation : foo"),
                Ok((" equation : foo", r"\%"))
            );
            assert_eq!(
                args_chunk_parser("equation* : foo"),
                Ok(("", "equation* : foo"))
            );
            assert_eq!(
                args_chunk_parser(r"$\mathcal{H}_2$"),
                Ok((r"\mathcal{H}_2$", "$"))
            );
            assert_eq!(
                args_chunk_parser(r"\textbf{\texttt{$\frac{1}{2}$}}"),
                Ok((r"textbf{\texttt{$\frac{1}{2}$}}", r"\"))
            );
        }
    }

    #[cfg(test)]
    mod args_parser_spec {
        use super::super::args_parser;

        #[test]
        fn should_take_whole_input() {
            for valid_input in args_parser_valid_input_with_escaped_chars_examples!() {
                assert_eq!(args_parser(valid_input), Ok(("", valid_input.to_string())));
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            for valid_input in args_parser_valid_input_with_escaped_chars_examples!() {
                let input = "%".to_string() + valid_input;
                assert_eq!(args_parser(&input), Ok((input.as_ref(), "".to_string())));
            }
        }

        #[test]
        fn should_stop_at_a_terminator_after_taking_as_much_as_possible() {
            for valid_input in args_parser_valid_input_with_escaped_chars_examples!() {
                let expected_rest = "%".to_string() + valid_input;
                let input_with_terminator = valid_input.to_string() + expected_rest.as_ref();
                assert_eq!(
                    args_parser(&input_with_terminator),
                    Ok((expected_rest.as_ref(), valid_input.to_string()))
                );
            }
        }

        #[test]
        fn realistic_examples() {
            assert_eq!(
                args_parser("equation: foo"),
                Ok((r"", "equation: foo".to_string()))
            );
            assert_eq!(
                args_parser("equation : foo"),
                Ok((r"", "equation : foo".to_string()))
            );
            assert_eq!(
                args_parser("equation [bar]: foo"),
                Ok((r"", "equation [bar]: foo".to_string()))
            );
            assert_eq!(
                args_parser("equation {bar}: foo"),
                Ok((r"", "equation {bar}: foo".to_string()))
            );
            assert_eq!(
                args_parser(r"equation {\bar\%}: foo"),
                Ok((r"", r"equation {\bar\%}: foo".to_string()))
            );
            assert_eq!(
                args_parser(r"equation {bar\: qux}: foo"),
                Ok((r"", r"equation {bar\: qux}: foo".to_string()))
            );
            assert_eq!(
                args_parser(r"\% equation : foo"),
                Ok(("", r"\% equation : foo".to_string()))
            );
            assert_eq!(
                args_parser(r"$\mathcal{H}_2$"),
                Ok(("", r"$\mathcal{H}_2$".to_string()))
            );
            assert_eq!(
                args_parser(r"\textbf{\texttt{$\frac{1}{2}$}}"),
                Ok(("", r"\textbf{\texttt{$\frac{1}{2}$}}".to_string()))
            );
        }
    }

    #[cfg(test)]
    mod other_tests {
        #[test]
        fn hashline_helper_plain_lines() {
            use super::super::{hashline_helper, Hashline};

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
            use super::super::{hashline_helper, Environment, Hashline};

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
            use super::super::{process_itemline, Hashline};

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
            use super::super::Environment;

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
            use super::super::list_env_parser;
            use nom::error::ErrorKind::Tag;
            use nom::Err::Error;

            assert_eq!(list_env_parser("itemize"), Ok(("", ())));
            assert_eq!(list_env_parser("enumerate*"), Ok(("*", ())));
            assert_eq!(list_env_parser("    description  *"), Ok(("  *", ())));
            assert_eq!(list_env_parser("item"), Err(Error(("item", Tag))));
            assert_eq!(list_env_parser("   foobar"), Err(Error(("foobar", Tag))));
        }
    }
}
// LCOV_EXCL_STOP
