use std::{num::ParseIntError, string::FromUtf8Error};

#[derive(Debug)]
pub enum Error {
    FromUtf8(FromUtf8Error),
    ParseInt(ParseIntError),
    InvalidBillVersion,
    MissingBillVersion,
    InvalidCongress,
    UnknownCongObjectType,
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUtf8(value)
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}
