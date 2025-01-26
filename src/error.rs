use std::{fmt::Display, num::ParseIntError, string::FromUtf8Error};

#[derive(Debug)]
pub enum Error {
    FromUtf8(FromUtf8Error),
    ParseInt(ParseIntError),
    InvalidBillVersion,
    MissingBillVersion,
    InvalidCongress,
    UnknownCongObjectType,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FromUtf8(e) => Display::fmt(e, f),
            Self::ParseInt(e) => Display::fmt(e, f),
            Self::InvalidBillVersion => f.write_str("not a valid bill version"),
            Self::MissingBillVersion => {
                f.write_str("url with bill version requested but no version given")
            }
            Self::InvalidCongress => {
                f.write_str("congress number in citation has not occurred yet")
            }
            Self::UnknownCongObjectType => {
                f.write_str("unknown or unsupported congressional object type")
            }
        }
    }
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

impl std::error::Error for Error {}
