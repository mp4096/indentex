// These are slightly modified parsers from nom/src/character.rs
// They have been modified to consume single bytes instead of characters

// Strictly speaking, `nom::ErrorKind::Char` is misleading, but we do not use it anyway
#[macro_export]
macro_rules! specific_byte (
  ($i:expr, $byte: expr) => (
    {
      if $i.is_empty() {
        nom::IResult::Incomplete::<_, _>(nom::Needed::Size(1))
      } else {
        if $byte == $i[0] {
          nom::IResult::Done(&$i[1..], $i[0])
        } else {
          nom::IResult::Error(error_position!(nom::ErrorKind::Char, $i))
        }
      }
    }
  );
);

#[macro_export]
macro_rules! none_of_bytes_as_bytes (
  ($i:expr, $bytes: expr) => (
    {
      if $i.is_empty() {
        nom::IResult::Incomplete::<_, _>(nom::Needed::Size(1))
      } else {
        let mut found = false;

        for &i in $bytes {
          if i == $i[0] {
            found = true;
            break;
          }
        }

        if !found {
          nom::IResult::Done(&$i[1..], $i[0])
        } else {
          nom::IResult::Error(error_position!(nom::ErrorKind::NoneOf, $i))
        }
      }
    }
  );
);


#[cfg(test)]
mod tests {
    use nom;
    use nom::IResult::{Done, Error};
    use nom::ErrorKind;

    #[test]
    fn none_of_bytes_as_bytes() {
        named!(f<u8>, none_of_bytes_as_bytes!("ab".as_bytes()));

        let a = &b"abcd"[..];
        assert_eq!(f(a), Error(error_position!(ErrorKind::NoneOf, a)));

        let b = &b"cde"[..];
        assert_eq!(f(b), Done(&b"de"[..], 'c' as u8));
    }

    #[test]
    fn specific_byte() {
        named!(f<u8>, specific_byte!('c' as u8));

        let a = &b"abcd"[..];
        assert_eq!(f(a), Error(error_position!(ErrorKind::Char, a)));

        let b = &b"cde"[..];
        assert_eq!(f(b), Done(&b"de"[..], 'c' as u8));
    }
}
