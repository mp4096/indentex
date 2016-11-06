// These are slightly modified parsers from nom/src/character.rs
// They have been modified to consume single bytes instead of characters

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
          nom::IResult::Error(error_position!(nom::ErrorKind::OneOf, $i))
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
