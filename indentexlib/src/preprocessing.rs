use std::io::{BufRead, Error};

pub fn read_and_trim_lines<T: BufRead>(buffered_reader: T) -> Result<Vec<String>, Error> {
    buffered_reader
        .lines()
        .map(|r| Ok(crate::utils::trim_end_inplace(r?)))
        .collect()
}
