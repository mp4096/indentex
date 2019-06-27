use std::io::{BufRead, Error};

pub fn read_and_trim_lines<T: BufRead>(buffered_reader: T) -> Result<Vec<String>, Error> {
    buffered_reader
        .lines()
        .map(|r| Ok(crate::utils::trim_end_inplace(r?)))
        .collect()
}

// LCOV_EXCL_START
#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod read_and_trim_lines_test {
        use super::super::read_and_trim_lines;
        use std::io::BufReader;

        #[test]
        fn valid_string() {
            let input = "\tfoo\nbar \n qux\t\n \t\n";

            let res = read_and_trim_lines(BufReader::new(input.as_bytes()));
            assert!(res.is_ok());

            let res_unwrapped = res.unwrap();
            assert_eq!(res_unwrapped.len(), 4);

            for (actual, expected) in res_unwrapped.iter().zip(vec!["\tfoo", "bar", " qux", ""]) {
                assert_eq!(actual, expected);
            }
        }

        #[test]
        fn invalid_utf8_string() {
            let input = b"\xe2\x28\xa1";
            let res = read_and_trim_lines(BufReader::new(input.as_ref()));
            assert!(res.is_err());
        }
    }
}
// LCOV_EXCL_STOP
