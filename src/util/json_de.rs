use std::{iter, slice};
use std::io::{Cursor, Error as IOError, Read};
use std::io::ErrorKind::InvalidData;
use std::io::Result as IOResult;

use serde::de::DeserializeOwned;
use serde_json::Deserializer;

fn read_skipping_ws(mut reader: impl Read) -> IOResult<u8> {
    loop {
        let mut byte = 0u8;
        reader.read_exact(slice::from_mut(&mut byte))?;
        if !byte.is_ascii_whitespace() {
            return Ok(byte);
        }
    }
}

fn invalid_data(msg: &str) -> IOError {
    IOError::new(InvalidData, msg)
}

fn deserialize_single<T, R>(reader: R) -> IOResult<T>
    where T: DeserializeOwned,
          R: Read
{
    let next_obj = Deserializer::from_reader(reader).into_iter::<T>().next();
    match next_obj {
        Some(result) => result.map_err(Into::into),
        None => Err(invalid_data("premature EOF")),
    }
}

fn yield_next_obj<T, R>(mut reader: R, at_start: &mut bool) -> IOResult<Option<T>>
    where T: DeserializeOwned,
          R: Read
{
    if !*at_start {
        *at_start = true;
        if read_skipping_ws(&mut reader)? == b'[' {
            // read the next char to see if the array is empty
            let peek = read_skipping_ws(&mut reader)?;
            if peek == b']' {
                Ok(None)
            } else {
                deserialize_single(Cursor::new([peek]).chain(reader)).map(Some)
            }
        } else {
            Err(invalid_data("`[` not found"))
        }
    } else {
        match read_skipping_ws(&mut reader)? {
            b',' => deserialize_single(reader).map(Some),
            b']' => Ok(None),
            _ => Err(invalid_data("`,` or `]` not found")),
        }
    }
}

pub fn iter_json_array<T, R>(mut reader: R) -> impl Iterator<Item=Result<T, IOError>>
    where T: DeserializeOwned,
          R: Read,
{
    let mut at_start = false;
    iter::from_fn(move || yield_next_obj(&mut reader, &mut at_start).transpose())
}