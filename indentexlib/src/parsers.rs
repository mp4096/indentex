use crate::parsing_types::{Hashline, RawHashlineParseData, RawItemlineParseData};

#[inline]
fn escaped_colon(input: &str) -> nom::IResult<&str, &str> {
    use nom::bytes::complete::tag;
    use nom::sequence::preceded;

    preceded(tag(r"\"), tag(":"))(input)
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

    alt((escaped_colon, tag(r"\%"), tag(r"\"), is_not(r"\:%")))(input)
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

    alt((tag(r"\%"), tag(r"\"), is_not(r"\%")))(input)
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

fn hashline_parser(input: &str) -> nom::IResult<&str, RawHashlineParseData> {
    use crate::utils::trim_end_inplace;
    use nom::bytes::complete::{is_a, tag};
    use nom::combinator::{opt, rest};

    let (input, indentation) = opt(is_a(" "))(input)?;
    let (input, _) = tag("# ")(input)?;
    let (input, name) = name_parser(input)?;
    let (input, _) = opt(is_a(" \t"))(input)?;
    let (input, opts) = opts_parser(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = opt(is_a(" \t"))(input)?;
    let (input, args) = args_parser(input)?;
    let (input, comment) = rest(input)?;

    // Name parser stops at whitespaces, so we assert that it is already trimmed
    debug_assert_eq!(&name, name.trim());
    // Opts and args parser consumes whitespaces, however due to a consuming whitespace parser
    // _before it_ we can assume that at least its result does not _start_ with a whitespace
    debug_assert_eq!(&opts, opts.trim_start());
    debug_assert_eq!(&args, args.trim_start());
    // Comment is either empty or always starts with a percent sign and is thus automatically
    // trimmed in the beginning
    debug_assert_eq!(comment, comment.trim_start());

    Ok((
        input,
        RawHashlineParseData {
            indent_depth: indentation.map_or(0, |s| s.len()),
            name,
            opts: trim_end_inplace(opts),
            args: trim_end_inplace(args),
            comment: comment.trim_end().to_string(),
        },
    ))
}

// Itemline parsers
fn itemline_parser(input: &str) -> nom::IResult<&str, RawItemlineParseData> {
    use nom::bytes::complete::{is_a, tag};
    use nom::combinator::{opt, rest};

    let (input, indentation) = opt(is_a(" "))(input)?;
    let (input, _) = tag("*")(input)?;
    let (input, item) = rest(input)?;

    Ok((
        input,
        RawItemlineParseData {
            indent_depth: indentation.map_or(0, |s| s.len()),
            item: item.trim().to_string(),
        },
    ))
}

// Itemline processing
#[inline]
fn process_itemline(line: String) -> Hashline {
    use self::Hashline::PlainLine;

    match itemline_parser(line.as_ref()) {
        Ok((_, r)) => r.into(),
        Err(_) => PlainLine(line),
    }
}

// Fully process line
pub fn process_line(line: String, list_like_active: bool) -> Hashline {
    use self::Hashline::PlainLine;

    match (hashline_parser(line.as_ref()), list_like_active) {
        (Ok((_, r)), _) => r.into(),
        (_, true) => process_itemline(line),
        (_, false) => PlainLine(line),
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
                assert_eq!(name_parser(input), Ok(("", input.replace(r"\:", ":"))));
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
            assert_eq!(opts_chunk_parser(r"\: foo"), Ok((" foo", ":")));
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
                assert_eq!(
                    opts_parser(valid_input),
                    Ok(("", valid_input.replace(r"\:", ":")))
                );
            }
        }

        #[test]
        fn should_stop_at_a_terminator_at_the_beginning() {
            for terminator in ":%".chars() {
                for valid_input in opts_parser_valid_input_with_escaped_chars_examples!() {
                    let input = terminator.to_string() + valid_input;
                    assert_eq!(opts_parser(&input), Ok((input.as_ref(), "".to_string())));
                }
            }
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
                }
            }
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
    mod hashline_parser_spec {
        use super::super::hashline_parser;

        #[test]
        fn valid_hashlines() {
            use super::super::RawHashlineParseData;

            for (input, expected_raw_parse_data) in vec![
                (
                    "# foo:      ",
                    RawHashlineParseData {
                        indent_depth: 0,
                        name: "foo".to_string(),
                        opts: "".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    " # foo: bar   ",
                    RawHashlineParseData {
                        indent_depth: 1,
                        name: "foo".to_string(),
                        opts: "".to_string(),
                        args: "bar".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    "  # foo[bar]:",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: "[bar]".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    "   # foo[bar]: qux",
                    RawHashlineParseData {
                        indent_depth: 3,
                        name: "foo".to_string(),
                        opts: "[bar]".to_string(),
                        args: "qux".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    r"    # foo[\:]: bar",
                    RawHashlineParseData {
                        indent_depth: 4,
                        name: "foo".to_string(),
                        opts: "[:]".to_string(),
                        args: "bar".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    "   # foo: % bar",
                    RawHashlineParseData {
                        indent_depth: 3,
                        name: "foo".to_string(),
                        opts: "".to_string(),
                        args: "".to_string(),
                        comment: "% bar".to_string(),
                    },
                ),
                (
                    "  # foo: bar % baz",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: "".to_string(),
                        args: "bar".to_string(),
                        comment: "% baz".to_string(),
                    },
                ),
                (
                    r" # foo: bar\% % baz   ",
                    RawHashlineParseData {
                        indent_depth: 1,
                        name: "foo".to_string(),
                        opts: "".to_string(),
                        args: r"bar\%".to_string(),
                        comment: "% baz".to_string(),
                    },
                ),
                (
                    r"# foo\:bar:",
                    RawHashlineParseData {
                        indent_depth: 0,
                        name: "foo:bar".to_string(),
                        opts: "".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    " # foo_bar:",
                    RawHashlineParseData {
                        indent_depth: 1,
                        name: "foo_bar".to_string(),
                        opts: "".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    "  # foo bar:",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: "bar".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    r"  # foo\bar:",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: r"\bar".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    r"  # foo \bar :",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: r"\bar".to_string(),
                        args: "".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    r"  # foo \bar :  qux   ",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: r"\bar".to_string(),
                        args: "qux".to_string(),
                        comment: "".to_string(),
                    },
                ),
                (
                    r"  # foo \bar :  qux   % blup    ",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: r"\bar".to_string(),
                        args: "qux".to_string(),
                        comment: "% blup".to_string(),
                    },
                ),
                (
                    "  # foo \tbar\t : \t qux \t  % \t blup  \t  ",
                    RawHashlineParseData {
                        indent_depth: 2,
                        name: "foo".to_string(),
                        opts: r"bar".to_string(),
                        args: "qux".to_string(),
                        comment: "% \t blup".to_string(),
                    },
                ),
            ] {
                assert_eq!(hashline_parser(input), Ok(("", expected_raw_parse_data)));
            }
        }

        #[test]
        fn not_hashlines_incorrect_begin() {
            use nom::error::ErrorKind::Tag;
            use nom::Err::Error;

            for (input, expected_rest) in vec![
                (" \t# foo:", "\t# foo:"), // consume whitespace, but stopped at the tab
                (r" \#", r"\#"),           // consume whitespace, but stopped at the backslash
                ("#foo:", "#foo:"),        // could not consume "# "
            ] {
                assert_eq!(hashline_parser(input), Err(Error((expected_rest, Tag))));
            }
        }

        #[test]
        fn not_hashlines_name_not_found() {
            use nom::error::ErrorKind::Many1;
            use nom::Err::Error;

            for (input, expected_rest) in vec![
                (" #  foo:", " foo:"), // consume "# " and stop immediately at the second whitespace
                ("# [foo:", "[foo:"),
            ] {
                assert_eq!(hashline_parser(input), Err(Error((expected_rest, Many1))));
            }
        }

        #[test]
        fn not_hashlines_colon_not_found() {
            use nom::error::ErrorKind::Tag;
            use nom::Err::Error;

            for input in vec!["# foo", "  # foo bar", r"  # foo \%    \:", "# #"] {
                assert_eq!(hashline_parser(input), Err(Error(("", Tag))));
            }
        }
    }

    #[cfg(test)]
    mod itemline_parser_spec {
        use super::super::itemline_parser;

        #[test]
        fn valid_itemlines() {
            use super::super::RawItemlineParseData;

            for (input, expected_raw_parse_data) in vec![
                (
                    "*",
                    RawItemlineParseData {
                        indent_depth: 0,
                        item: "".to_string(),
                    },
                ),
                (
                    "*  ",
                    RawItemlineParseData {
                        indent_depth: 0,
                        item: "".to_string(),
                    },
                ),
                (
                    "  *",
                    RawItemlineParseData {
                        indent_depth: 2,
                        item: "".to_string(),
                    },
                ),
                (
                    "  *  ",
                    RawItemlineParseData {
                        indent_depth: 2,
                        item: "".to_string(),
                    },
                ),
                (
                    "*foo",
                    RawItemlineParseData {
                        indent_depth: 0,
                        item: "foo".to_string(),
                    },
                ),
                (
                    "* foo",
                    RawItemlineParseData {
                        indent_depth: 0,
                        item: "foo".to_string(),
                    },
                ),
                (
                    "   * bar",
                    RawItemlineParseData {
                        indent_depth: 3,
                        item: "bar".to_string(),
                    },
                ),
                (
                    "***",
                    RawItemlineParseData {
                        indent_depth: 0,
                        item: "**".to_string(),
                    },
                ),
            ] {
                assert_eq!(itemline_parser(input), Ok(("", expected_raw_parse_data)));
            }
        }

        #[test]
        fn not_itemlines() {
            use nom::error::ErrorKind::Tag;
            use nom::Err::Error;

            for (input, expected_rest) in vec![
                ("   baz   ", "baz   "),
                ("qux   *", "qux   *"),
                ("  abc * def", "abc * def"),
                (r"  \\*  ", r"\\*  "),
                (r"  \*  ", r"\*  "),
                (r"\*  ", r"\*  "),
            ] {
                assert_eq!(itemline_parser(input), Err(Error((expected_rest, Tag))));
            }
        }
    }

    #[cfg(test)]
    mod process_line_spec {
        use super::super::process_line;

        #[test]
        fn yield_single_line_command() {
            use super::super::Hashline::PlainLine;

            for is_a_list_environment in vec![true, false] {
                for (input, expected_result) in vec![
                    (" # foo: bar", r" \foo{bar}"),
                    ("  # foo{qux}: bar", r"  \foo{qux}{bar}"),
                    ("  # foo    [qux]: bar", r"  \foo[qux]{bar}"),
                    ("  # section*: bar", r"  \section*{bar}"),
                    ("# section[qux]: bar", r"\section[qux]{bar}"),
                    ("  # foo: bar % baz", r"  \foo{bar} % baz"),
                    ("  # foo: bar      % baz", r"  \foo{bar} % baz"),
                    ("  # foo[qux]: bar      % baz", r"  \foo[qux]{bar} % baz"),
                    (r"  # foo[\:qux]: bar   % baz", r"  \foo[:qux]{bar} % baz"),
                    (r"  # foo: bar \% % baz", r"  \foo{bar \%} % baz"),
                    (r"  # foo: bar \%% baz", r"  \foo{bar \%} % baz"),
                ] {
                    assert_eq!(
                        process_line(input.to_string(), is_a_list_environment),
                        PlainLine(expected_result.to_string())
                    );
                }
            }
        }

        #[test]
        fn yield_plainline_outside_a_list_environment() {
            use super::super::Hashline::PlainLine;

            for s in vec![
                "foo bar 123",
                "  * 123 foo bar",
                r"$\frac{1}{2}",
                r"\begin{equation}",
                r"\end{equation}",
                r"\textbf{1}",
                r"\newcommand{\foo}[1]{\textrm{#1}}",
                "",
                " ",
                "\t",
                " * foo",
                r"  \\",
            ] {
                assert_eq!(process_line(s.to_string(), false), PlainLine(s.to_string()));
            }
        }

        #[test]
        fn yield_itemline() {
            use super::super::Hashline::PlainLine;

            for (input, expected_result) in vec![
                ("* foo bar 123", r"\item foo bar 123"),
                ("  * 123 foo bar", r"  \item 123 foo bar"),
                (r"   * $\frac{1}{2}", r"   \item $\frac{1}{2}"),
                (r"* \textbf{1}", r"\item \textbf{1}"),
                ("  *", r"  \item"),
                ("  *[A] B", r"  \item [A] B"),
            ] {
                assert_eq!(
                    process_line(input.to_string(), true),
                    PlainLine(expected_result.to_string())
                );
            }
        }

        #[test]
        fn yield_plainline_in_a_list_environment() {
            use super::super::Hashline::PlainLine;

            for s in vec![
                r"\item foo bar 123",
                "   123 foo bar",
                r"    $\frac{1}{2}",
                r" \textbf{1}",
                r"  \\",
                "  ",
                "  [A] B",
            ] {
                assert_eq!(process_line(s.to_string(), true), PlainLine(s.to_string()));
            }
        }

        #[test]
        fn yield_environment() {
            use super::super::Hashline::OpenEnv;
            use crate::parsing_types::Environment;

            for is_a_list_environment in vec![true, false] {
                for (input, expected_result) in vec![
                    (
                        " # foo: ",
                        Environment::new(
                            1,
                            "foo".to_string(),
                            "".to_string(),
                            "".to_string(),
                            false,
                        ),
                    ),
                    (
                        "  # foo{qux}:",
                        Environment::new(
                            2,
                            "foo".to_string(),
                            "{qux}".to_string(),
                            "".to_string(),
                            false,
                        ),
                    ),
                    (
                        "  # foo    [qux]: %",
                        Environment::new(
                            2,
                            "foo".to_string(),
                            "[qux]".to_string(),
                            "%".to_string(),
                            false,
                        ),
                    ),
                    (
                        "  # equation*:  % bar",
                        Environment::new(
                            2,
                            "equation*".to_string(),
                            "".to_string(),
                            "% bar".to_string(),
                            false,
                        ),
                    ),
                    (
                        "# equation*      :  ",
                        Environment::new(
                            0,
                            "equation*".to_string(),
                            "".to_string(),
                            "".to_string(),
                            false,
                        ),
                    ),
                    (
                        "  # itemize:   % baz",
                        Environment::new(
                            2,
                            "itemize".to_string(),
                            "".to_string(),
                            "% baz".to_string(),
                            true,
                        ),
                    ),
                    (
                        r"   # foo[\:qux]:% baz",
                        Environment::new(
                            3,
                            "foo".to_string(),
                            "[:qux]".to_string(),
                            "% baz".to_string(),
                            false,
                        ),
                    ),
                ] {
                    assert_eq!(
                        process_line(input.to_string(), is_a_list_environment),
                        OpenEnv(expected_result),
                    );
                }
            }
        }
    }
}
// LCOV_EXCL_STOP
